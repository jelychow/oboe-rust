package com.example.openairustrealtime.core.audio

import com.google.oboe.AudioApi
import com.google.oboe.AudioDirection
import com.google.oboe.AudioFormat
import com.google.oboe.AudioStream
import com.google.oboe.AudioStreamBuilder
import com.google.oboe.PerformanceMode
import java.util.Locale
import java.util.concurrent.atomic.AtomicBoolean

class RealtimeOboeAudioPump(
    private val onInputAudio: (FloatArray, Int) -> Unit,
    private val pollOutputAudio: () -> ByteArray?,
    private val onOutputAudio: (Int, Float) -> Unit,
    private val onDiagnostics: (RealtimeAudioDiagnostics) -> Unit,
    private val onError: (String) -> Unit
) {
    private val stopRequested = AtomicBoolean(false)
    private val diagnosticsLock = Any()
    @Volatile private var inputThread: Thread? = null
    @Volatile private var outputThread: Thread? = null
    @Volatile private var inputStream: AudioStream? = null
    @Volatile private var outputStream: AudioStream? = null
    private var latestInputDiagnostics = StreamDiagnostics()
    private var latestOutputDiagnostics = StreamDiagnostics()
    private var latestOutputLatencyMillis = 0f
    private var latestOutputPendingFrames = 0L
    private var lastAsyncError = 0

    @Synchronized
    fun start() {
        stop()
        stopRequested.set(false)
        inputThread = Thread(::runInputPump, "oboe-sdk-realtime-input").also { it.start() }
        outputThread = Thread(::runOutputPump, "oboe-sdk-realtime-output").also { it.start() }
    }

    @Synchronized
    fun stop() {
        stopRequested.set(true)
        runCatching { inputStream?.requestStop() }
        runCatching { outputStream?.requestStop() }
        inputThread?.join(500L)
        outputThread?.join(500L)
        inputThread = null
        outputThread = null
        inputStream = null
        outputStream = null
    }

    private fun runInputPump() {
        while (!stopRequested.get()) {
            var stream: AudioStream? = null
            try {
                stream = openStream(AudioDirection.INPUT)
                inputStream = stream
                val runtime = configureLowLatency(stream, AudioDirection.INPUT)
                check(stream.requestStart() == 0) { "Oboe realtime input stream failed to start." }
                sampleDiagnostics(stream, runtime, force = true)
                val buffer = FloatArray(PcmAudio.FRAMES_PER_CHUNK * PcmAudio.CHANNEL_COUNT)
                while (!stopRequested.get()) {
                    val read = stream.readFloat(buffer, 0, buffer.size, PcmAudio.IO_TIMEOUT_NANOS)
                    if (read < 0) {
                        throw RecoverableAudioStreamError(
                            "Oboe realtime input read failed with code $read.",
                            read
                        )
                    }
                    sampleDiagnostics(stream, runtime)
                    if (read == 0) {
                        Thread.sleep(IDLE_SLEEP_MILLIS)
                        continue
                    }
                    onInputAudio(buffer, read)
                }
                stream.requestStop()
                return
            } catch (error: RecoverableAudioStreamError) {
                if (!stopRequested.get()) {
                    publishAsyncError(error.code)
                    sleepBeforeReopen()
                }
            } catch (error: Throwable) {
                if (!stopRequested.get()) onError(error.message ?: error.toString())
                return
            } finally {
                inputStream = null
                runCatching { stream?.close() }
            }
        }
    }

    private fun runOutputPump() {
        while (!stopRequested.get()) {
            var stream: AudioStream? = null
            try {
                stream = openStream(AudioDirection.OUTPUT)
                outputStream = stream
                val runtime = configureLowLatency(stream, AudioDirection.OUTPUT)
                check(stream.requestStart() == 0) { "Oboe realtime output stream failed to start." }
                sampleDiagnostics(stream, runtime, force = true)
                while (!stopRequested.get()) {
                    val pcm = pollOutputAudio()
                    if (pcm == null || pcm.isEmpty()) {
                        sampleDiagnostics(stream, runtime)
                        Thread.sleep(IDLE_SLEEP_MILLIS)
                        continue
                    }
                    val audio = PcmAudio.pcm16ToFloatArray(pcm)
                    writeFully(stream, runtime, audio)
                    onOutputAudio(
                        audio.size / PcmAudio.CHANNEL_COUNT,
                        PcmAudio.audioLevel(audio, audio.size)
                    )
                    sampleDiagnostics(stream, runtime)
                }
                stream.requestStop()
                return
            } catch (error: RecoverableAudioStreamError) {
                if (!stopRequested.get()) {
                    publishAsyncError(error.code)
                    sleepBeforeReopen()
                }
            } catch (error: Throwable) {
                if (!stopRequested.get()) onError(error.message ?: error.toString())
                return
            } finally {
                outputStream = null
                runCatching { stream?.close() }
            }
        }
    }

    private fun openStream(direction: AudioDirection): AudioStream {
        return AudioStreamBuilder()
            .setAudioApi(AudioApi.AAUDIO)
            .setDirection(direction)
            .setSampleRate(PcmAudio.SAMPLE_RATE)
            .setChannelCount(PcmAudio.CHANNEL_COUNT)
            .setFormat(AudioFormat.FLOAT)
            .setPerformanceMode(PerformanceMode.LOW_LATENCY)
            .openStream()
    }

    private fun configureLowLatency(stream: AudioStream, direction: AudioDirection): StreamRuntime {
        val burstFrames = positiveOrZero { stream.getFramesPerBurst() }
        val capacityFrames = positiveOrZero { stream.getBufferCapacityInFrames() }
        val requestedFrames = if (burstFrames > 0) burstFrames * INITIAL_BUFFER_BURSTS else 0
        val tunedFrames = if (requestedFrames > 0) {
            positiveOrZero { stream.setBufferSizeInFrames(requestedFrames) }
        } else {
            0
        }
        val bufferFrames = positiveOrZero { stream.getBufferSizeInFrames() }
            .ifZero(tunedFrames)
            .ifZero(capacityFrames)
        val runtime = StreamRuntime(
            direction = direction,
            burstFrames = burstFrames,
            bufferCapacityFrames = capacityFrames,
            bufferSizeFrames = bufferFrames,
            lastXRunCount = positiveOrZero { stream.getXRunCount() }
        )
        publishDiagnostics(runtime, outputLatencyMillis = 0f, outputPendingFrames = 0L, asyncError = 0)
        return runtime
    }

    private fun writeFully(stream: AudioStream, runtime: StreamRuntime, audio: FloatArray) {
        var offset = 0
        while (!stopRequested.get() && offset < audio.size) {
            val written = stream.writeFloat(
                audio,
                offset,
                audio.size - offset,
                PcmAudio.IO_TIMEOUT_NANOS
            )
            if (written < 0) {
                throw RecoverableAudioStreamError(
                    "Oboe realtime output write failed with code $written.",
                    written
                )
            }
            check(written > 0) { "Oboe realtime output write returned zero frames." }
            offset += written
            sampleDiagnostics(stream, runtime)
        }
    }

    private fun sampleDiagnostics(
        stream: AudioStream,
        runtime: StreamRuntime,
        force: Boolean = false
    ) {
        val now = System.nanoTime()
        if (!force && now < runtime.nextSampleAtNanos) return
        runtime.nextSampleAtNanos = now + DIAGNOSTIC_SAMPLE_INTERVAL_NANOS

        val asyncError = stream.getAndClearLastError()
        if (asyncError < 0) {
            publishDiagnostics(runtime, stream, asyncError)
            throw RecoverableAudioStreamError(
                "Oboe realtime ${runtime.label} stream async error $asyncError.",
                asyncError
            )
        }

        val xrunCount = positiveOrZero { stream.getXRunCount() }
        if (xrunCount > runtime.lastXRunCount) {
            retuneBufferAfterXRun(stream, runtime)
        } else {
            runtime.bufferSizeFrames = positiveOrZero { stream.getBufferSizeInFrames() }
                .ifZero(runtime.bufferSizeFrames)
        }
        runtime.lastXRunCount = xrunCount
        publishDiagnostics(runtime, stream, asyncError)
    }

    private fun retuneBufferAfterXRun(stream: AudioStream, runtime: StreamRuntime) {
        val burstFrames = runtime.burstFrames
        if (burstFrames <= 0) return
        val currentFrames = positiveOrZero { stream.getBufferSizeInFrames() }
            .ifZero(runtime.bufferSizeFrames)
            .ifZero(burstFrames * INITIAL_BUFFER_BURSTS)
        val capacityFrames = runtime.bufferCapacityFrames.ifZero(positiveOrZero {
            stream.getBufferCapacityInFrames()
        })
        if (runtime.bufferCapacityFrames == 0 && capacityFrames > 0) {
            runtime.bufferCapacityFrames = capacityFrames
        }
        val targetFrames = if (capacityFrames > 0) {
            (currentFrames + burstFrames).coerceAtMost(capacityFrames)
        } else {
            currentFrames + burstFrames
        }
        if (targetFrames > currentFrames) {
            runtime.bufferSizeFrames = positiveOrZero { stream.setBufferSizeInFrames(targetFrames) }
                .ifZero(currentFrames)
        }
    }

    private fun publishDiagnostics(
        runtime: StreamRuntime,
        stream: AudioStream,
        asyncError: Int
    ) {
        val outputPendingFrames = if (runtime.direction == AudioDirection.OUTPUT) {
            outputPendingFrames(stream)
        } else {
            0L
        }
        val outputLatencyMillis = if (runtime.direction == AudioDirection.OUTPUT) {
            framesToMillis(outputPendingFrames)
        } else {
            0f
        }
        publishDiagnostics(runtime, outputLatencyMillis, outputPendingFrames, asyncError)
    }

    private fun publishDiagnostics(
        runtime: StreamRuntime,
        outputLatencyMillis: Float,
        outputPendingFrames: Long,
        asyncError: Int
    ) {
        val streamDiagnostics = StreamDiagnostics(
            xRunCount = runtime.lastXRunCount,
            burstFrames = runtime.burstFrames,
            bufferSizeFrames = runtime.bufferSizeFrames,
            bufferCapacityFrames = runtime.bufferCapacityFrames
        )
        val snapshot = synchronized(diagnosticsLock) {
            if (runtime.direction == AudioDirection.INPUT) {
                latestInputDiagnostics = streamDiagnostics
            } else {
                latestOutputDiagnostics = streamDiagnostics
                latestOutputLatencyMillis = outputLatencyMillis
                latestOutputPendingFrames = outputPendingFrames
            }
            if (asyncError != 0) lastAsyncError = asyncError
            RealtimeAudioDiagnostics(
                inputXRunCount = latestInputDiagnostics.xRunCount,
                outputXRunCount = latestOutputDiagnostics.xRunCount,
                inputBurstFrames = latestInputDiagnostics.burstFrames,
                outputBurstFrames = latestOutputDiagnostics.burstFrames,
                inputBufferSizeFrames = latestInputDiagnostics.bufferSizeFrames,
                outputBufferSizeFrames = latestOutputDiagnostics.bufferSizeFrames,
                inputBufferCapacityFrames = latestInputDiagnostics.bufferCapacityFrames,
                outputBufferCapacityFrames = latestOutputDiagnostics.bufferCapacityFrames,
                outputLatencyMillis = latestOutputLatencyMillis,
                outputPendingFrames = latestOutputPendingFrames,
                lastAsyncError = lastAsyncError
            )
        }
        onDiagnostics(snapshot)
    }

    private fun publishAsyncError(errorCode: Int) {
        val snapshot = synchronized(diagnosticsLock) {
            if (errorCode != 0) lastAsyncError = errorCode
            RealtimeAudioDiagnostics(
                inputXRunCount = latestInputDiagnostics.xRunCount,
                outputXRunCount = latestOutputDiagnostics.xRunCount,
                inputBurstFrames = latestInputDiagnostics.burstFrames,
                outputBurstFrames = latestOutputDiagnostics.burstFrames,
                inputBufferSizeFrames = latestInputDiagnostics.bufferSizeFrames,
                outputBufferSizeFrames = latestOutputDiagnostics.bufferSizeFrames,
                inputBufferCapacityFrames = latestInputDiagnostics.bufferCapacityFrames,
                outputBufferCapacityFrames = latestOutputDiagnostics.bufferCapacityFrames,
                outputLatencyMillis = latestOutputLatencyMillis,
                outputPendingFrames = latestOutputPendingFrames,
                lastAsyncError = lastAsyncError
            )
        }
        onDiagnostics(snapshot)
    }

    private fun outputPendingFrames(stream: AudioStream): Long {
        return runCatching {
            val timestamp = stream.getTimestamp()
            val framesWritten = stream.getFramesWritten()
            (framesWritten - timestamp.framePosition).coerceAtLeast(0L)
        }.getOrDefault(0L)
    }

    private fun framesToMillis(frames: Long): Float {
        if (frames <= 0L) return 0f
        return frames.toFloat() * MILLIS_PER_SECOND / PcmAudio.SAMPLE_RATE.toFloat()
    }

    private fun positiveOrZero(block: () -> Int): Int = runCatching(block).getOrDefault(0).coerceAtLeast(0)

    private fun Int.ifZero(fallback: Int): Int = if (this == 0) fallback else this

    private fun sleepBeforeReopen() {
        Thread.sleep(STREAM_REOPEN_BACKOFF_MILLIS)
    }

    private data class StreamRuntime(
        val direction: AudioDirection,
        val burstFrames: Int,
        var bufferCapacityFrames: Int,
        var bufferSizeFrames: Int,
        var lastXRunCount: Int,
        var nextSampleAtNanos: Long = 0L
    ) {
        val label: String
            get() = direction.name.lowercase(Locale.US)
    }

    private data class StreamDiagnostics(
        val xRunCount: Int = 0,
        val burstFrames: Int = 0,
        val bufferSizeFrames: Int = 0,
        val bufferCapacityFrames: Int = 0
    )

    private class RecoverableAudioStreamError(message: String, val code: Int) : RuntimeException(message)

    private companion object {
        private const val INITIAL_BUFFER_BURSTS = 2
        private const val DIAGNOSTIC_SAMPLE_INTERVAL_NANOS = 250_000_000L
        private const val STREAM_REOPEN_BACKOFF_MILLIS = 150L
        private const val IDLE_SLEEP_MILLIS = 5L
        private const val MILLIS_PER_SECOND = 1_000f
    }
}
