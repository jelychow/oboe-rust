package com.example.openairustrealtime.core.audio

import com.google.oboe.AudioApi
import com.google.oboe.AudioDirection
import com.google.oboe.AudioFormat
import com.google.oboe.AudioStream
import com.google.oboe.AudioStreamBuilder
import com.google.oboe.PerformanceMode
import java.util.concurrent.atomic.AtomicBoolean

class RealtimeOboeAudioPump(
    private val onInputAudio: (FloatArray, Int) -> Unit,
    private val pollOutputAudio: () -> ByteArray?,
    private val onOutputAudio: (Int, Float) -> Unit,
    private val onError: (String) -> Unit
) {
    private val stopRequested = AtomicBoolean(false)
    @Volatile private var inputThread: Thread? = null
    @Volatile private var outputThread: Thread? = null
    @Volatile private var inputStream: AudioStream? = null
    @Volatile private var outputStream: AudioStream? = null

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
        inputStream?.requestStop()
        outputStream?.requestStop()
        inputThread?.join(500L)
        outputThread?.join(500L)
        inputThread = null
        outputThread = null
        inputStream = null
        outputStream = null
    }

    private fun runInputPump() {
        var stream: AudioStream? = null
        try {
            stream = openStream(AudioDirection.INPUT)
            inputStream = stream
            check(stream.requestStart() == 0) { "Oboe realtime input stream failed to start." }
            val buffer = FloatArray(PcmAudio.FRAMES_PER_CHUNK * PcmAudio.CHANNEL_COUNT)
            while (!stopRequested.get()) {
                val read = stream.readFloat(buffer, 0, buffer.size, PcmAudio.IO_TIMEOUT_NANOS)
                check(read >= 0) { "Oboe realtime input read failed with code $read." }
                if (read == 0) {
                    Thread.sleep(5L)
                    continue
                }
                onInputAudio(buffer, read)
            }
            stream.requestStop()
        } catch (error: Throwable) {
            if (!stopRequested.get()) onError(error.message ?: error.toString())
        } finally {
            inputStream = null
            stream?.close()
        }
    }

    private fun runOutputPump() {
        var stream: AudioStream? = null
        try {
            stream = openStream(AudioDirection.OUTPUT)
            outputStream = stream
            check(stream.requestStart() == 0) { "Oboe realtime output stream failed to start." }
            while (!stopRequested.get()) {
                val pcm = pollOutputAudio()
                if (pcm == null || pcm.isEmpty()) {
                    Thread.sleep(5L)
                    continue
                }
                val audio = PcmAudio.pcm16ToFloatArray(pcm)
                writeFully(stream, audio)
                onOutputAudio(
                    audio.size / PcmAudio.CHANNEL_COUNT,
                    PcmAudio.audioLevel(audio, audio.size)
                )
            }
            stream.requestStop()
        } catch (error: Throwable) {
            if (!stopRequested.get()) onError(error.message ?: error.toString())
        } finally {
            outputStream = null
            stream?.close()
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

    private fun writeFully(stream: AudioStream, audio: FloatArray) {
        var offset = 0
        while (!stopRequested.get() && offset < audio.size) {
            val written = stream.writeFloat(
                audio,
                offset,
                audio.size - offset,
                PcmAudio.IO_TIMEOUT_NANOS
            )
            check(written > 0) { "Oboe realtime output write failed with code $written." }
            offset += written
        }
    }
}
