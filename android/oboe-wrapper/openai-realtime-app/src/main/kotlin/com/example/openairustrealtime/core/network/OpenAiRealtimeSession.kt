package com.example.openairustrealtime.core.network

import android.util.Log
import com.example.openairustrealtime.core.util.AppLog
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
import java.net.HttpURLConnection
import java.net.URL
import java.net.URLEncoder
import java.util.Locale
import java.util.concurrent.TimeUnit
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CompletableDeferred
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
    private val bargeInController = RealtimeBargeInController()
    private val assistantPlaybackController = RealtimeAssistantPlaybackController()
    private var nextSessionId = 0L
    private var runningSession: RunningSession? = null
    private val snapshot = RealtimeSnapshot()

    fun start(apiKey: String, model: String, instructions: String): Int {
        val config = RealtimeConfig.create(apiKey, model, instructions) ?: return -1
        AppLog.i(TAG, "Realtime start requested model=${config.model}")
        verifyRealtimeAccess(config)
        AppLog.i(TAG, "Realtime REST preflight passed model=${config.model}")
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
            val controlChannel = Channel<RealtimeClientCommand>(CONTROL_CHANNEL_CAPACITY)
            val outputQueue = RealtimeOutputQueue()
            val ready = CompletableDeferred<Int>()
            val job = scope.launch {
                runRealtimeSession(id, config, audioChannel, controlChannel, outputQueue, ready)
            }
            AppLog.i(TAG, "Realtime session job created id=$id model=${config.model}")
            RunningSession(id, job, audioChannel, controlChannel, outputQueue, ready).also {
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
        val readyCode = runBlocking {
            withTimeoutOrNull(START_TIMEOUT_MS) {
                session.ready.await()
            }
        } ?: run {
            setError("Timed out waiting for Realtime session.updated.")
            -2
        }
        if (readyCode != 0) {
            Log.w(TAG, "Realtime session failed before ready id=${session.id} readyCode=$readyCode")
            session.audioChannel.close()
            session.controlChannel.close()
            runBlocking {
                session.job.cancelAndJoin()
            }
            throw RealtimeApiException(lastError().ifBlank { "Realtime session failed to connect." })
        }
        AppLog.i(TAG, "Realtime session ready id=${session.id}")
        return 0
    }

    fun stop(reason: String = "unspecified"): Int {
        val session = synchronized(lock) {
            reapFinishedLocked()
            runningSession.also {
                runningSession = null
                if (it != null) {
                    AppLog.i(TAG, "Realtime stop requested reason=$reason id=${it.id} status=${snapshot.status}")
                    setStatusLocked("Stopping")
                } else {
                    AppLog.d(TAG, "Realtime stop requested reason=$reason with no active session status=${snapshot.status}")
                }
            }
        }
        if (session != null) {
            session.audioChannel.close()
            session.controlChannel.close()
            runBlocking {
                session.job.cancelAndJoin()
            }
        }
        synchronized(lock) {
            AppLog.i(TAG, "Realtime stopped reason=$reason previousStatus=${snapshot.status}")
            setStatusLocked("Stopped")
        }
        return 0
    }

    fun appendInputAudio(audio: FloatArray, sampleCount: Int): Int {
        val copied = audio.copyOf(sampleCount.coerceIn(0, audio.size))
        val micLevel = PcmAudio.audioLevel(copied, copied.size)
        val session = synchronized(lock) {
            reapFinishedLocked()
            val activeSession = runningSession ?: return -1
            snapshot.inputLevel = micLevel.coerceIn(0f, 1f)
            val decision = bargeInController.evaluate(snapshot.status, snapshot.inputLevel)
            if (decision.shouldCancelResponse) {
                cancelAssistantOutputLocked(activeSession)
            }
            if (!decision.shouldUploadMic) {
                return 1
            }
            activeSession
        }

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
            runningSession?.outputQueue?.poll()
        }
    }

    fun shouldInterruptOutput(): Boolean = assistantPlaybackController.shouldAbortPlayback()

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
        stop("session.close")
        scope.cancel()
        client.close()
    }

    private suspend fun runRealtimeSession(
        sessionId: Long,
        config: RealtimeConfig,
        audioChannel: Channel<FloatArray>,
        controlChannel: Channel<RealtimeClientCommand>,
        outputQueue: RealtimeOutputQueue,
        ready: CompletableDeferred<Int>
    ) {
        try {
            client.webSocket({
                url("wss://api.openai.com/v1/realtime?model=${config.encodedModel}")
                header(HttpHeaders.Authorization, "Bearer ${config.apiKey}")
                header(HttpHeaders.ContentType, "application/json")
            }) {
                AppLog.i(TAG, "Realtime WebSocket connected id=$sessionId model=${config.model}")
                sendTextWithTimeout(RealtimeProtocol.sessionUpdate(config.model, config.instructions))
                AppLog.d(TAG, "Realtime session.update sent id=$sessionId")
                awaitSessionUpdated(outputQueue, ready)

                while (isActive) {
                    select<Unit> {
                        audioChannel.onReceiveCatching { result ->
                            val audio = result.getOrNull()
                            if (audio == null) {
                                AppLog.i(TAG, "Realtime audio channel closed by client id=$sessionId")
                                close(CloseReason(CloseReason.Codes.NORMAL, "client stopped"))
                                throw RealtimeClosed("client audio channel closed")
                            }
                            sendTextWithTimeout(RealtimeProtocol.audioAppend(audio, audio.size))
                            recordInputSent(audio)
                        }
                        controlChannel.onReceiveCatching { result ->
                            when (result.getOrNull()) {
                                RealtimeClientCommand.CANCEL_RESPONSE -> {
                                    AppLog.i(TAG, "Realtime barge-in cancel requested id=$sessionId")
                                    sendTextWithTimeout(RealtimeProtocol.responseCancel())
                                }
                                null -> Unit
                            }
                        }
                        incoming.onReceiveCatching { result ->
                            val frame = result.getOrNull() ?: throw RealtimeClosed("incoming channel closed during audio loop")
                            when (frame) {
                                is Frame.Text -> handleServerEvent(frame.readText(), outputQueue)
                                is Frame.Close -> throw RealtimeClosed("server close frame during audio loop")
                                else -> Unit
                            }
                        }
                    }
                }
            }
            setStoppedIfCurrent(sessionId)
        } catch (closed: RealtimeClosed) {
            AppLog.i(TAG, "Realtime session closed id=$sessionId reason=${closed.closeReason}")
            if (!ready.isCompleted) ready.complete(-2)
            setStoppedIfCurrent(sessionId)
        } catch (error: CancellationException) {
            throw error
        } catch (error: Throwable) {
            setError(error.message ?: error.toString())
            if (!ready.isCompleted) ready.complete(-2)
            Log.w(TAG, "Realtime session failed: ${lastError()}", error)
        } finally {
            if (!ready.isCompleted) ready.complete(-2)
            audioChannel.close()
            controlChannel.close()
            clearSessionIfCurrent(sessionId)
        }
    }

    private suspend fun DefaultClientWebSocketSession.awaitSessionUpdated(
        outputQueue: RealtimeOutputQueue,
        ready: CompletableDeferred<Int>
    ) {
        val updated = withTimeoutOrNull(SESSION_UPDATE_TIMEOUT_MS) {
            while (isActive) {
                val frame = incoming.receiveCatching().getOrNull()
                    ?: throw RealtimeClosed("incoming closed before session.updated")
                when (frame) {
                    is Frame.Text -> {
                        val text = frame.readText()
                        val type = runCatching { RealtimeProtocol.eventType(text) }.getOrDefault("")
                        AppLog.d(TAG, "Realtime handshake event type=${type.ifBlank { "(blank)" }}")
                        handleServerEvent(text, outputQueue)
                        if (type == "error") {
                            ready.complete(-2)
                            throw RealtimeClosed("server error during handshake")
                        }
                        if (type == "session.updated") {
                            AppLog.i(TAG, "Realtime session.updated received")
                            setStatus("Connected")
                            ready.complete(0)
                            return@withTimeoutOrNull true
                        }
                    }
                    is Frame.Close -> throw RealtimeClosed("server close frame before session.updated")
                    else -> Unit
                }
            }
            false
        }
        if (updated != true) {
            setError("Timed out waiting for Realtime session.updated.")
            ready.complete(-2)
            throw RealtimeClosed("timed out waiting for session.updated")
        }
    }

    private suspend fun DefaultClientWebSocketSession.sendTextWithTimeout(text: String) {
        val sent = withTimeoutOrNull(WS_SEND_TIMEOUT_MS) {
            send(text)
            true
        }
        check(sent == true) { "Timed out sending Realtime WebSocket event." }
    }

    private fun handleServerEvent(text: String, outputQueue: RealtimeOutputQueue) {
        val type = runCatching { RealtimeProtocol.eventType(text) }.getOrDefault("")
        logServerEvent(type)
        when (val event = runCatching { RealtimeProtocol.parseServerEvent(text) }.getOrElse {
            RealtimeServerEvent.Error("Invalid Realtime event JSON: ${it.message ?: it}")
        }) {
            is RealtimeServerEvent.AudioDelta -> {
                if (!assistantPlaybackController.shouldDropIncomingAudio()) {
                    outputQueue.offer(event.bytes)
                }
            }
            is RealtimeServerEvent.TranscriptDelta -> appendTranscript(event.text)
            is RealtimeServerEvent.StatusChanged -> {
                if (event.status == "Responding") {
                    assistantPlaybackController.onResponseStarted()
                }
                setStatus(event.status)
            }
            is RealtimeServerEvent.TranscriptDone -> {
                appendTranscript("\n")
                setStatus("Connected")
            }
            is RealtimeServerEvent.Error -> setError(event.message)
            RealtimeServerEvent.Ignored -> Unit
        }
    }

    private fun cancelAssistantOutputLocked(session: RunningSession) {
        assistantPlaybackController.requestInterrupt()
        session.outputQueue.clear()
        session.controlChannel.trySend(RealtimeClientCommand.CANCEL_RESPONSE)
        setStatusLocked("Interrupting")
    }

    private fun logServerEvent(type: String) {
        when (type) {
            "response.output_audio.delta",
            "response.audio.delta",
            "response.output_audio_transcript.delta",
            "response.audio_transcript.delta" -> Unit
            "" -> Log.w(TAG, "Realtime server event missing type")
            else -> AppLog.d(TAG, "Realtime server event type=$type")
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
        if (snapshot.status != status) AppLog.d(TAG, "Realtime status: $status")
        snapshot.status = status
        bargeInController.onStatusChanged(status)
        assistantPlaybackController.onStatusChanged(status)
    }

    private fun setError(error: String) {
        val redacted = SecretRedactor.redact(error)
        Log.w(TAG, "Realtime error: $redacted")
        synchronized(lock) {
            snapshot.status = "Error"
            snapshot.lastError = redacted
            bargeInController.onStatusChanged("Error")
            assistantPlaybackController.onStatusChanged("Error")
        }
    }

    private fun verifyRealtimeAccess(config: RealtimeConfig) {
        val connection = (URL("$BASE_URL/models/${config.encodedModel}").openConnection() as HttpURLConnection).apply {
            requestMethod = "GET"
            connectTimeout = CONNECT_TIMEOUT_MS.toInt()
            readTimeout = CONNECT_TIMEOUT_MS.toInt()
            setRequestProperty(HttpHeaders.Authorization, "Bearer ${config.apiKey}")
        }
        try {
            val code = connection.responseCode
            AppLog.d(TAG, "Realtime REST preflight HTTP $code model=${config.model}")
            if (code !in 200..299) {
                val body = connection.errorStream?.use { it.readBytes() } ?: ByteArray(0)
                throw RealtimeApiException(formatApiError(code, body))
            }
        } finally {
            connection.disconnect()
        }
    }

    private fun formatApiError(code: Int, body: ByteArray): String {
        val raw = body.toString(Charsets.UTF_8).trim()
        val message = runCatching {
            org.json.JSONObject(raw).optJSONObject("error")
                ?.optString("message")
                ?.takeIf { it.isNotBlank() }
        }.getOrNull() ?: raw.ifBlank { "request failed" }
        return SecretRedactor.redact("OpenAI API $code: $message")
    }

    private fun setStoppedIfCurrent(sessionId: Long) {
        synchronized(lock) {
            if (runningSession?.id == sessionId && snapshot.status != "Error") {
                AppLog.i(TAG, "Realtime native session reached stopped id=$sessionId previousStatus=${snapshot.status}")
                setStatusLocked("Stopped")
            }
        }
    }

    private fun clearSessionIfCurrent(sessionId: Long) {
        synchronized(lock) {
            if (runningSession?.id == sessionId) {
                AppLog.d(TAG, "Realtime clearing current session id=$sessionId")
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
        val controlChannel: Channel<RealtimeClientCommand>,
        val outputQueue: RealtimeOutputQueue,
        val ready: CompletableDeferred<Int>
    )

    private enum class RealtimeClientCommand {
        CANCEL_RESPONSE
    }

    private class RealtimeClosed(val closeReason: String) : Throwable(closeReason)

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
        private const val TAG = "OpenAiRealtime"
        private const val BASE_URL = "https://api.openai.com/v1"
        private const val MIC_AUDIO_QUEUE_CAPACITY = 8
        private const val CONTROL_CHANNEL_CAPACITY = 4
        private const val WS_SEND_TIMEOUT_MS = 5_000L
        private const val START_TIMEOUT_MS = 20_000L
        private const val SESSION_UPDATE_TIMEOUT_MS = 10_000L
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

class RealtimeApiException(message: String) : Exception(message)
