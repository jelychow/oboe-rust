package com.example.openairustrealtime.core.audio

import com.google.oboe.AudioApi
import com.google.oboe.AudioDirection
import com.google.oboe.AudioFormat
import com.google.oboe.AudioStream
import com.google.oboe.AudioStreamBuilder
import com.google.oboe.PerformanceMode
import java.util.concurrent.atomic.AtomicBoolean

class OboePcmPlayer(
    private val onProgress: (frames: Int, level: Float) -> Unit,
    private val onError: (String) -> Unit = {},
    private val onFinished: () -> Unit = {}
) {
    private val stopRequested = AtomicBoolean(false)
    @Volatile private var playbackThread: Thread? = null
    @Volatile private var activeStream: AudioStream? = null

    @Synchronized
    fun playPcm16(pcm: ByteArray) {
        if (pcm.isEmpty()) {
            throw IllegalArgumentException("PCM audio is empty.")
        }
        stop()
        stopRequested.set(false)
        val audio = PcmAudio.pcm16ToFloatArray(pcm)
        playbackThread = Thread({
            var stream: AudioStream? = null
            try {
                stream = openOutputStream()
                activeStream = stream
                check(stream.requestStart() == 0) { "Oboe output stream failed to start." }
                var offset = 0
                val chunkSamples = PcmAudio.FRAMES_PER_CHUNK * PcmAudio.CHANNEL_COUNT
                while (!stopRequested.get() && offset < audio.size) {
                    val sampleCount = minOf(chunkSamples, audio.size - offset)
                    writeFully(stream, audio, offset, sampleCount)
                    onProgress(
                        sampleCount / PcmAudio.CHANNEL_COUNT,
                        PcmAudio.audioLevel(audio, offset, sampleCount)
                    )
                    offset += sampleCount
                }
                stream.requestStop()
            } catch (error: Throwable) {
                if (!stopRequested.get()) onError(error.message ?: error.toString())
            } finally {
                activeStream = null
                stream?.close()
                onFinished()
            }
        }, "oboe-sdk-pcm-player").also { it.start() }
    }

    @Synchronized
    fun stop() {
        stopRequested.set(true)
        activeStream?.requestStop()
        playbackThread?.join(500L)
        playbackThread = null
        activeStream = null
    }

    private fun openOutputStream(): AudioStream {
        return AudioStreamBuilder()
            .setAudioApi(AudioApi.AAUDIO)
            .setDirection(AudioDirection.OUTPUT)
            .setSampleRate(PcmAudio.SAMPLE_RATE)
            .setChannelCount(PcmAudio.CHANNEL_COUNT)
            .setFormat(AudioFormat.FLOAT)
            .setPerformanceMode(PerformanceMode.LOW_LATENCY)
            .openStream()
    }

    private fun writeFully(stream: AudioStream, audio: FloatArray, offset: Int, sampleCount: Int) {
        var writtenTotal = 0
        while (writtenTotal < sampleCount && !stopRequested.get()) {
            val written = stream.writeFloat(
                audio,
                offset + writtenTotal,
                sampleCount - writtenTotal,
                PcmAudio.IO_TIMEOUT_NANOS
            )
            check(written > 0) { "Oboe output write failed with code $written." }
            writtenTotal += written
        }
    }
}
