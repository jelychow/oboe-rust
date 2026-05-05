package com.example.openairustrealtime.core.audio

import android.content.Context
import android.util.Log
import com.example.openairustrealtime.core.util.AppLog
import com.google.oboe.AudioDirection
import com.google.oboe.AudioStream
import com.google.oboe.AudioStreamBuilder
import java.util.Locale
import java.util.concurrent.atomic.AtomicBoolean

class RealtimeOboeAudioPump(
    context: Context,
    private val onInputAudio: (FloatArray, Int) -> Int,
    private val pollOutputAudio: () -> ByteArray?,
    private val shouldInterruptOutput: () -> Boolean,
    private val onOutputAudio: (frames: Int, level: Float) -> Unit,
    private val onDiagnostics: (RealtimeAudioDiagnostics) -> Unit,
    private val onError: (String) -> Unit
) {
    private val audioEffects = RealtimeAudioEffects(context.applicationContext)
    private val streamOpenLock = Any()
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
        AppLog.i(TAG, "Realtime audio pump start requested")
        stop("restart.beforeStart")
        audioEffects.enterCommunicationMode()
        stopRequested.set(false)
        inputThread = Thread(::runInputPump, "oboe-sdk-realtime-input").also { it.start() }
        outputThread = Thread(::runOutputPump, "oboe-sdk-realtime-output").also { it.start() }
    }

    @Synchronized
    fun stop(reason: String = "unspecified") {
        AppLog.i(TAG, "Realtime audio pump stop requested reason=$reason")
        stopRequested.set(true)
        runCatching { inputStream?.requestStop() }
        runCatching { outputStream?.requestStop() }
        inputThread?.join(500L)
        outputThread?.join(500L)
        inputThread = null
        outputThread = null
        inputStream = null
        outputStream = null
        audioEffects.leaveCommunicationMode()
    }

    private fun runInputPump() {
        AppLog.i(TAG, "Realtime input pump thread started")
        val recovery = RealtimeStreamRecoveryPolicy(MAX_STREAM_REOPEN_ATTEMPTS)
        while (!stopRequested.get()) {
            var stream: AudioStream? = null
            var captureEffects: RealtimeAudioEffects.CaptureEffects? = null
            try {
                stream = openStream(AudioDirection.INPUT)
                inputStream = stream
                recovery.recordStreamOpened()
                val runtime = configureLowLatency(stream, AudioDirection.INPUT)
                AppLog.i(TAG, "Realtime input stream opened ${runtime.summary()}")
                captureEffects = audioEffects.attachCaptureEffects(stream)
                check(stream.requestStart() == 0) { "Oboe realtime input stream failed to start." }
                AppLog.i(TAG, "Realtime input stream started")
                sampleDiagnostics(stream, runtime, force = true)
                val buffer = FloatArray(PcmAudio.FRAMES_PER_CHUNK * PcmAudio.CHANNEL_COUNT)
                while (!stopRequested.get()) {
                    val read = stream.readFloat(buffer, 0, buffer.size, PcmAudio.REALTIME_IO_TIMEOUT_NANOS)
                    if (read < 0) {
                        throw RecoverableAudioStreamError(
                            "Oboe realtime input read failed with code $read.",
                            read
                        )
                    }
                    sampleDiagnostics(stream, runtime)
                    if (read == 0) {
                        continue
                    }
                    onInputAudio(buffer, read)
                    recovery.recordSuccessfulAudioIo()
                }
                stream.requestStop()
                return
            } catch (error: RecoverableAudioStreamError) {
                if (!stopRequested.get()) {
                    Log.w(TAG, error.message ?: "Recoverable input stream error")
                    if (!recovery.recordRecoverableFailure()) {
                        onError("Input stream failed after $MAX_STREAM_REOPEN_ATTEMPTS recovery attempts.")
                        return
                    }
                    publishAsyncError(error.code)
                    sleepBeforeReopen()
                }
            } catch (error: Throwable) {
                if (!stopRequested.get()) {
                    Log.w(TAG, "Realtime input pump failed", error)
                    onError(error.message ?: error.toString())
                }
                return
            } finally {
                runCatching { captureEffects?.close() }
                inputStream = null
                runCatching { stream?.close() }
                AppLog.i(TAG, "Realtime input stream closed stopRequested=${stopRequested.get()}")
            }
        }
    }

    private fun runOutputPump() {
        AppLog.i(TAG, "Realtime output pump thread started")
        val recovery = RealtimeStreamRecoveryPolicy(MAX_STREAM_REOPEN_ATTEMPTS)
        while (!stopRequested.get()) {
            var stream: AudioStream? = null
            try {
                stream = openStream(AudioDirection.OUTPUT)
                outputStream = stream
                recovery.recordStreamOpened()
                val runtime = configureLowLatency(stream, AudioDirection.OUTPUT)
                AppLog.i(TAG, "Realtime output stream opened ${runtime.summary()}")
                check(stream.requestStart() == 0) { "Oboe realtime output stream failed to start." }
                AppLog.i(TAG, "Realtime output stream started")
                sampleDiagnostics(stream, runtime, force = true)
                while (!stopRequested.get()) {
                    val pcm = pollOutputAudio()
                    if (pcm == null || pcm.isEmpty()) {
                        sampleDiagnostics(stream, runtime)
                        Thread.sleep(OUTPUT_IDLE_SLEEP_MILLIS)
                        continue
                    }
                    val audio = PcmAudio.pcm16ToFloatArray(pcm)
                    val writtenSamples = writeFully(stream, runtime, audio)
                    recovery.recordSuccessfulAudioIo()
                    if (writtenSamples > 0) {
                        onOutputAudio(
                            writtenSamples / PcmAudio.CHANNEL_COUNT,
                            PcmAudio.audioLevel(audio, writtenSamples)
                        )
                    }
                    sampleDiagnostics(stream, runtime)
                }
                stream.requestStop()
                return
            } catch (error: RecoverableAudioStreamError) {
                if (!stopRequested.get()) {
                    Log.w(TAG, error.message ?: "Recoverable output stream error")
                    if (!recovery.recordRecoverableFailure()) {
                        onError("Output stream failed after $MAX_STREAM_REOPEN_ATTEMPTS recovery attempts.")
                        return
                    }
                    publishAsyncError(error.code)
                    sleepBeforeReopen()
                }
            } catch (error: Throwable) {
                if (!stopRequested.get()) {
                    Log.w(TAG, "Realtime output pump failed", error)
                    onError(error.message ?: error.toString())
                }
                return
            } finally {
                outputStream = null
                runCatching { stream?.close() }
                AppLog.i(TAG, "Realtime output stream closed stopRequested=${stopRequested.get()}")
            }
        }
    }

    private fun openStream(direction: AudioDirection): AudioStream {
        val failures = mutableListOf<String>()
        val label = direction.name.lowercase(Locale.US)
        for (spec in RealtimeAudioStreamOpenPlan.specsFor(direction)) {
            try {
                val stream = synchronized(streamOpenLock) {
                    openStream(spec)
                }
                AppLog.i(TAG, "Realtime $label stream opened spec=${spec.label}")
                return stream
            } catch (error: Throwable) {
                val message = error.message ?: error.toString()
                Log.w(TAG, "Realtime $label stream open failed spec=${spec.label}: $message")
                failures += "${spec.label}: $message"
            }
        }
        throw IllegalStateException(
            "Realtime $label stream open failed for all configs: ${failures.joinToString("; ")}"
        )
    }

    private fun openStream(spec: RealtimeAudioStreamOpenSpec): AudioStream {
        val builder = AudioStreamBuilder()
            .setAudioApi(spec.audioApi)
            .setDirection(spec.direction)
            .setSampleRate(spec.sampleRate)
            .setChannelCount(spec.channelCount)
            .setFormat(spec.format)
            .setPerformanceMode(spec.performanceMode)
        if (spec.direction == AudioDirection.INPUT && spec.useVoiceCommunicationInput) {
            RealtimeOboeCompat.configureVoiceCommunicationInput(
                builder = builder,
                allocateSession = spec.allocateInputSession
            )
        } else if (spec.direction == AudioDirection.OUTPUT && spec.useVoiceCommunicationOutput) {
            RealtimeOboeCompat.configureVoiceCommunicationOutput(builder)
        }
        return builder.openStream()
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

    private fun writeFully(stream: AudioStream, runtime: StreamRuntime, audio: FloatArray): Int {
        var offset = 0
        val zeroProgressPolicy = RealtimeZeroProgressPolicy(MAX_ZERO_PROGRESS_WRITES)
        while (!stopRequested.get() && offset < audio.size) {
            if (shouldInterruptOutput()) {
                break
            }
            val writeSize = minOf(audio.size - offset, PcmAudio.FRAMES_PER_CHUNK * PcmAudio.CHANNEL_COUNT)
            val written = stream.writeFloat(
                audio,
                offset,
                writeSize,
                PcmAudio.REALTIME_IO_TIMEOUT_NANOS
            )
            if (written < 0) {
                throw RecoverableAudioStreamError(
                    "Oboe realtime output write failed with code $written.",
                    written
                )
            }
            if (written == 0) {
                sampleDiagnostics(stream, runtime)
                if (!zeroProgressPolicy.recordZeroProgressWrite()) {
                    throw RecoverableAudioStreamError(
                        "Oboe realtime output write made no progress.",
                        0
                    )
                }
                Thread.sleep(ZERO_PROGRESS_SLEEP_MILLIS)
                continue
            }
            zeroProgressPolicy.recordSuccessfulWrite()
            offset += written
            sampleDiagnostics(stream, runtime)
        }
        return offset
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
            runtime.stableSampleCount = 0
            retuneBufferAfterXRun(stream, runtime)
        } else {
            runtime.stableSampleCount++
            if (runtime.stableSampleCount >= SHRINK_STABLE_SAMPLES && runtime.burstFrames > 0) {
                shrinkBufferIfPossible(stream, runtime)
            }
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

    private fun shrinkBufferIfPossible(stream: AudioStream, runtime: StreamRuntime) {
        val burstFrames = runtime.burstFrames
        val floorFrames = burstFrames * INITIAL_BUFFER_BURSTS
        val currentFrames = positiveOrZero { stream.getBufferSizeInFrames() }
            .ifZero(runtime.bufferSizeFrames)
        if (currentFrames <= floorFrames) return
        val targetFrames = (currentFrames - burstFrames).coerceAtLeast(floorFrames)
        val actual = positiveOrZero { stream.setBufferSizeInFrames(targetFrames) }
        if (actual > 0) {
            runtime.bufferSizeFrames = actual
            runtime.stableSampleCount = 0
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
        var stableSampleCount: Int = 0,
        var nextSampleAtNanos: Long = 0L
    ) {
        val label: String
            get() = direction.name.lowercase(Locale.US)

        fun summary(): String {
            return "direction=$label burst=$burstFrames buffer=$bufferSizeFrames/$bufferCapacityFrames xruns=$lastXRunCount"
        }
    }

    private data class StreamDiagnostics(
        val xRunCount: Int = 0,
        val burstFrames: Int = 0,
        val bufferSizeFrames: Int = 0,
        val bufferCapacityFrames: Int = 0
    )

    private class RecoverableAudioStreamError(message: String, val code: Int) : RuntimeException(message)

    private companion object {
        private const val TAG = "RealtimeOboeAudioPump"
        private const val INITIAL_BUFFER_BURSTS = 2
        private const val DIAGNOSTIC_SAMPLE_INTERVAL_NANOS = 250_000_000L
        private const val STREAM_REOPEN_BACKOFF_MILLIS = 150L
        private const val MAX_STREAM_REOPEN_ATTEMPTS = 5
        private const val MAX_ZERO_PROGRESS_WRITES = 4
        private const val SHRINK_STABLE_SAMPLES = 8
        private const val OUTPUT_IDLE_SLEEP_MILLIS = 2L
        private const val ZERO_PROGRESS_SLEEP_MILLIS = 2L
        private const val MILLIS_PER_SECOND = 1_000f
    }
}
