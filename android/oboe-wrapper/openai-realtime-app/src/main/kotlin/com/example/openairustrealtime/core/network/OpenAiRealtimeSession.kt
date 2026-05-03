package com.example.openairustrealtime.core.network

import com.example.openairustrealtime.core.audio.PcmAudio
import com.example.openairustrealtime.core.audio.RealtimeAudioDiagnostics
import com.example.openairustrealtime.core.util.SecretRedactor
import io.ktor.client.HttpClient
import io.ktor.client.engine.okhttp.OkHttp
import io.ktor.client.plugins.websocket.DefaultClientWebSocketSession
import io.ktor.client.plugins.websocket.WebSockets
import io.ktor.client.plugins.websocket.webSocket
import io.ktor.client.request.header
import io.ktor.client.request.url
import io.ktor.http.HttpHeaders
import io.ktor.websocket.CloseReason
import io.ktor.websocket.Frame
import io.ktor.websocket.close
import io.ktor.websocket.readText
import io.ktor.websocket.send
import java.io.Closeable
import java.net.URLEncoder
import java.util.ArrayDeque
import java.util.Locale
import java.util.concurrent.TimeUnit
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.selects.select
import kotlinx.coroutines.withTimeoutOrNull

class OpenAiRealtimeSession(
    private val client: HttpClient = defaultClient()
) : Closeable {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val lock = Any()
    private var nextSessionId = 0L
    private var runningSession: RunningSession? = null
    private val snapshot = RealtimeSnapshot()

    fun start(apiKey: String, model: String, instructions: String): Int {
        val config = RealtimeConfig.create(apiKey, model, instructions) ?: return -1
        val session = synchronized(lock) {
            reapFinishedLocked()
            val active = runningSession
            if (active != null && active.job.isActive) {
                setStatusLocked("Already running")
                return 0
            }

            snapshot.reset()
            setStatusLocked("Connecting")
            val id = allocateSessionIdLocked()
            val audioChannel = Channel<FloatArray>(MIC_AUDIO_QUEUE_CAPACITY)
            val outputQueue = ArrayDeque<ByteArray>()
            val job = scope.launch {
                runRealtimeSession(id, config, audioChannel, outputQueue)
            }
            RunningSession(id, job, audioChannel, outputQueue).also {
                runningSession = it
            }
        }

        session.job.invokeOnCompletion {
            synchronized(lock) {
                if (runningSession?.id == session.id && !session.job.isActive) {
                    runningSession = null
                }
            }
        }
        return 0
    }

    fun stop(): Int {
        val session = synchronized(lock) {
            reapFinishedLocked()
            runningSession.also {
                runningSession = null
                if (it != null) setStatusLocked("Stopping")
            }
        }
        if (session != null) {
            session.audioChannel.close()
            runBlocking {
                session.job.cancelAndJoin()
            }
        }
        synchronized(lock) {
            snapshot.status = "Stopped"
        }
        return 0
    }

    fun appendInputAudio(audio: FloatArray, sampleCount: Int): Int {
        val copied = audio.copyOf(sampleCount.coerceIn(0, audio.size))
        val session = synchronized(lock) {
            reapFinishedLocked()
            runningSession
        } ?: return -1

        return if (session.audioChannel.trySend(copied).isSuccess) {
            0
        } else {
            synchronized(lock) {
                snapshot.droppedInputChunks = snapshot.droppedInputChunks.saturatingInc()
                snapshot.droppedInputFrames = snapshot.droppedInputFrames.saturatingAdd(copied.size.toLong())
            }
            -2
        }
    }

    fun pollOutputAudio(): ByteArray? {
        return synchronized(lock) {
            reapFinishedLocked()
            runningSession?.outputQueue?.pollFirst()
        }
    }

    fun recordOutputAudio(frames: Int, level: Float) {
        synchronized(lock) {
            snapshot.outputChunks = snapshot.outputChunks.saturatingInc()
            snapshot.outputFrames = snapshot.outputFrames.saturatingAdd(frames.coerceAtLeast(0).toLong())
            snapshot.outputLevel = level.coerceIn(0f, 1f)
        }
    }

    fun recordAudioDiagnostics(diagnostics: RealtimeAudioDiagnostics) {
        synchronized(lock) {
            snapshot.inputXRunCount = diagnostics.inputXRunCount.coerceAtLeast(0)
            snapshot.outputXRunCount = diagnostics.outputXRunCount.coerceAtLeast(0)
            snapshot.inputBurstFrames = diagnostics.inputBurstFrames.coerceAtLeast(0)
            snapshot.outputBurstFrames = diagnostics.outputBurstFrames.coerceAtLeast(0)
            snapshot.inputBufferSizeFrames = diagnostics.inputBufferSizeFrames.coerceAtLeast(0)
            snapshot.outputBufferSizeFrames = diagnostics.outputBufferSizeFrames.coerceAtLeast(0)
            snapshot.inputBufferCapacityFrames = diagnostics.inputBufferCapacityFrames.coerceAtLeast(0)
            snapshot.outputBufferCapacityFrames = diagnostics.outputBufferCapacityFrames.coerceAtLeast(0)
            snapshot.outputLatencyMillis = diagnostics.outputLatencyMillis.coerceAtLeast(0f)
            snapshot.outputPendingFrames = diagnostics.outputPendingFrames.coerceAtLeast(0L)
            snapshot.lastAsyncError = diagnostics.lastAsyncError
        }
    }

    fun reportAudioError(message: String) {
        setError(message)
    }

    fun status(): String = synchronized(lock) {
        reapFinishedLocked()
        snapshot.status
    }

    fun transcript(): String = synchronized(lock) { snapshot.transcript }

    fun lastError(): String = synchronized(lock) { snapshot.lastError }

    fun stats(): String = synchronized(lock) {
        "Mic sent: ${snapshot.inputChunksSent} chunks / ${snapshot.inputFramesSent} frames. " +
            "Mic dropped: ${snapshot.droppedInputChunks} chunks / ${snapshot.droppedInputFrames} frames. " +
            "Output played: ${snapshot.outputChunks} chunks / ${snapshot.outputFrames} frames. " +
            "Levels: mic ${String.format(Locale.US, "%.3f", snapshot.inputLevel.coerceIn(0f, 1f))}, " +
            "output ${String.format(Locale.US, "%.3f", snapshot.outputLevel.coerceIn(0f, 1f))}. " +
            "Diagnostics: xruns input ${snapshot.inputXRunCount} / output ${snapshot.outputXRunCount}. " +
            "Output latency ${String.format(Locale.US, "%.1f", snapshot.outputLatencyMillis)} ms / " +
            "${snapshot.outputPendingFrames} frames pending. " +
            "Buffer: input ${snapshot.inputBufferSizeFrames}/${snapshot.inputBufferCapacityFrames} " +
            "burst ${snapshot.inputBurstFrames}, output " +
            "${snapshot.outputBufferSizeFrames}/${snapshot.outputBufferCapacityFrames} " +
            "burst ${snapshot.outputBurstFrames}. Async error: ${snapshot.lastAsyncError}."
    }

    override fun close() {
        stop()
        scope.cancel()
        client.close()
    }

    private suspend fun runRealtimeSession(
        sessionId: Long,
        config: RealtimeConfig,
        audioChannel: Channel<FloatArray>,
        outputQueue: ArrayDeque<ByteArray>
    ) {
        try {
            client.webSocket({
                url("wss://api.openai.com/v1/realtime?model=${config.encodedModel}")
                header(HttpHeaders.Authorization, "Bearer ${config.apiKey}")
                header(HttpHeaders.ContentType, "application/json")
            }) {
                setStatus("Connected")
                sendTextWithTimeout(RealtimeProtocol.sessionUpdate(config.model, config.instructions))

                while (isActive) {
                    select<Unit> {
                        audioChannel.onReceiveCatching { result ->
                            val audio = result.getOrNull()
                            if (audio == null) {
                                close(CloseReason(CloseReason.Codes.NORMAL, "client stopped"))
                                throw RealtimeClosed()
                            }
                            sendTextWithTimeout(RealtimeProtocol.audioAppend(audio, audio.size))
                            recordInputSent(audio)
                        }
                        incoming.onReceiveCatching { result ->
                            val frame = result.getOrNull() ?: throw RealtimeClosed()
                            when (frame) {
                                is Frame.Text -> handleServerEvent(frame.readText(), outputQueue)
                                is Frame.Close -> throw RealtimeClosed()
                                else -> Unit
                            }
                        }
                    }
                }
            }
            setStoppedIfCurrent(sessionId)
        } catch (_: RealtimeClosed) {
            setStoppedIfCurrent(sessionId)
        } catch (error: CancellationException) {
            throw error
        } catch (error: Throwable) {
            setError(error.message ?: error.toString())
        } finally {
            audioChannel.close()
            clearSessionIfCurrent(sessionId)
        }
    }

    private suspend fun DefaultClientWebSocketSession.sendTextWithTimeout(text: String) {
        val sent = withTimeoutOrNull(WS_SEND_TIMEOUT_MS) {
            send(text)
            true
        }
        check(sent == true) { "Timed out sending Realtime WebSocket event." }
    }

    private fun handleServerEvent(text: String, outputQueue: ArrayDeque<ByteArray>) {
        when (val event = runCatching { RealtimeProtocol.parseServerEvent(text) }.getOrElse {
            RealtimeServerEvent.Error("Invalid Realtime event JSON: ${it.message ?: it}")
        }) {
            is RealtimeServerEvent.AudioDelta -> synchronized(lock) {
                while (outputQueue.size >= OUTPUT_AUDIO_QUEUE_CAPACITY) {
                    outputQueue.pollFirst()
                }
                outputQueue.addLast(event.bytes)
            }
            is RealtimeServerEvent.TranscriptDelta -> appendTranscript(event.text)
            is RealtimeServerEvent.StatusChanged -> setStatus(event.status)
            is RealtimeServerEvent.TranscriptDone -> {
                appendTranscript("\n")
                setStatus("Connected")
            }
            is RealtimeServerEvent.Error -> setError(event.message)
            RealtimeServerEvent.Ignored -> Unit
        }
    }

    private fun recordInputSent(audio: FloatArray) {
        synchronized(lock) {
            snapshot.inputChunksSent = snapshot.inputChunksSent.saturatingInc()
            snapshot.inputFramesSent = snapshot.inputFramesSent.saturatingAdd(audio.size.toLong())
            snapshot.inputLevel = PcmAudio.audioLevel(audio, audio.size)
        }
    }

    private fun appendTranscript(delta: String) {
        synchronized(lock) {
            snapshot.transcript += delta
            if (snapshot.transcript.length > MAX_TRANSCRIPT_CHARS) {
                snapshot.transcript = snapshot.transcript.takeLast(MAX_TRANSCRIPT_CHARS)
            }
        }
    }

    private fun setStatus(status: String) {
        synchronized(lock) {
            setStatusLocked(status)
        }
    }

    private fun setStatusLocked(status: String) {
        snapshot.status = status
    }

    private fun setError(error: String) {
        synchronized(lock) {
            snapshot.status = "Error"
            snapshot.lastError = SecretRedactor.redact(error)
        }
    }

    private fun setStoppedIfCurrent(sessionId: Long) {
        synchronized(lock) {
            if (runningSession?.id == sessionId && snapshot.status != "Error") {
                snapshot.status = "Stopped"
            }
        }
    }

    private fun clearSessionIfCurrent(sessionId: Long) {
        synchronized(lock) {
            if (runningSession?.id == sessionId) {
                runningSession = null
            }
        }
    }

    private fun reapFinishedLocked() {
        if (runningSession?.job?.isCompleted == true) {
            runningSession = null
        }
    }

    private fun allocateSessionIdLocked(): Long {
        nextSessionId = if (nextSessionId == Long.MAX_VALUE) 1L else nextSessionId + 1L
        return nextSessionId
    }

    private data class RunningSession(
        val id: Long,
        val job: Job,
        val audioChannel: Channel<FloatArray>,
        val outputQueue: ArrayDeque<ByteArray>
    )

    private class RealtimeClosed : Throwable()

    private data class RealtimeConfig(
        val apiKey: String,
        val model: String,
        val instructions: String
    ) {
        val encodedModel: String = URLEncoder.encode(model, "UTF-8")

        companion object {
            fun create(apiKey: String, model: String, instructions: String): RealtimeConfig? {
                val normalizedKey = apiKey.trim()
                if (normalizedKey.isBlank()) return null
                return RealtimeConfig(
                    apiKey = normalizedKey,
                    model = model.trim().ifBlank { "gpt-realtime" },
                    instructions = instructions.trim().ifBlank {
                        "You are a concise realtime voice assistant. Reply in the user's language."
                    }
                )
            }
        }
    }

    private data class RealtimeSnapshot(
        var status: String = "Stopped",
        var transcript: String = "",
        var lastError: String = "",
        var inputChunksSent: Long = 0L,
        var inputFramesSent: Long = 0L,
        var droppedInputChunks: Long = 0L,
        var droppedInputFrames: Long = 0L,
        var outputChunks: Long = 0L,
        var outputFrames: Long = 0L,
        var inputLevel: Float = 0f,
        var outputLevel: Float = 0f,
        var inputXRunCount: Int = 0,
        var outputXRunCount: Int = 0,
        var inputBurstFrames: Int = 0,
        var outputBurstFrames: Int = 0,
        var inputBufferSizeFrames: Int = 0,
        var outputBufferSizeFrames: Int = 0,
        var inputBufferCapacityFrames: Int = 0,
        var outputBufferCapacityFrames: Int = 0,
        var outputLatencyMillis: Float = 0f,
        var outputPendingFrames: Long = 0L,
        var lastAsyncError: Int = 0
    ) {
        fun reset() {
            status = "Connecting"
            transcript = ""
            lastError = ""
            inputChunksSent = 0L
            inputFramesSent = 0L
            droppedInputChunks = 0L
            droppedInputFrames = 0L
            outputChunks = 0L
            outputFrames = 0L
            inputLevel = 0f
            outputLevel = 0f
            inputXRunCount = 0
            outputXRunCount = 0
            inputBurstFrames = 0
            outputBurstFrames = 0
            inputBufferSizeFrames = 0
            outputBufferSizeFrames = 0
            inputBufferCapacityFrames = 0
            outputBufferCapacityFrames = 0
            outputLatencyMillis = 0f
            outputPendingFrames = 0L
            lastAsyncError = 0
        }
    }

    private companion object {
        private const val MIC_AUDIO_QUEUE_CAPACITY = 8
        private const val OUTPUT_AUDIO_QUEUE_CAPACITY = 32
        private const val WS_SEND_TIMEOUT_MS = 5_000L
        private const val CONNECT_TIMEOUT_MS = 15_000L
        private const val MAX_TRANSCRIPT_CHARS = 8_192

        fun defaultClient(): HttpClient {
            return HttpClient(OkHttp) {
                install(WebSockets)
                engine {
                    config {
                        connectTimeout(CONNECT_TIMEOUT_MS, TimeUnit.MILLISECONDS)
                        readTimeout(0L, TimeUnit.MILLISECONDS)
                        writeTimeout(WS_SEND_TIMEOUT_MS, TimeUnit.MILLISECONDS)
                    }
                }
            }
        }

        fun Long.saturatingInc(): Long = if (this == Long.MAX_VALUE) Long.MAX_VALUE else this + 1L

        fun Long.saturatingAdd(value: Long): Long {
            if (value <= 0L) return this
            return if (Long.MAX_VALUE - this < value) Long.MAX_VALUE else this + value
        }
    }
}
