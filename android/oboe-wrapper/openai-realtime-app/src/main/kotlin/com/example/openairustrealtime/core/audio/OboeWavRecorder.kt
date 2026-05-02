package com.example.openairustrealtime.core.audio

import com.google.oboe.AudioApi
import com.google.oboe.AudioDirection
import com.google.oboe.AudioFormat
import com.google.oboe.AudioStream
import com.google.oboe.AudioStreamBuilder
import com.google.oboe.PerformanceMode
import java.io.File
import java.io.RandomAccessFile
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicReference

class OboeWavRecorder(
    private val onProgress: (frames: Int, level: Float) -> Unit
) {
    private data class Running(
        val file: File,
        val stop: AtomicBoolean,
        val error: AtomicReference<Throwable?>,
        val thread: Thread
    )

    @Volatile private var running: Running? = null

    @Synchronized
    fun start(file: File): File {
        check(running == null) { "Recording is already running." }
        val stop = AtomicBoolean(false)
        val error = AtomicReference<Throwable?>(null)
        val thread = Thread({
            runCatching { recordToWav(file, stop) }
                .onFailure { throwable -> error.set(throwable) }
        }, "oboe-sdk-wav-recorder")
        running = Running(file, stop, error, thread)
        thread.start()
        return file
    }

    @Synchronized
    fun stop(): File {
        val current = running ?: error("No recording is active.")
        current.stop.set(true)
        current.thread.join()
        running = null
        current.error.get()?.let { error ->
            throw IllegalStateException(error.message ?: error.toString(), error)
        }
        check(current.file.exists() && current.file.length() > PcmAudio.WAV_HEADER_BYTES) {
            "Oboe SDK recording did not produce microphone samples."
        }
        return current.file
    }

    @Synchronized
    fun cancel() {
        val current = running ?: return
        current.stop.set(true)
        current.thread.join(500L)
        running = null
    }

    private fun recordToWav(file: File, stop: AtomicBoolean) {
        var stream: AudioStream? = null
        RandomAccessFile(file, "rw").use { output ->
            output.setLength(0L)
            output.write(ByteArray(PcmAudio.WAV_HEADER_BYTES.toInt()))
            var dataBytes = 0L
            try {
                val openedStream = openInputStream()
                stream = openedStream
                check(openedStream.requestStart() == 0) { "Oboe input stream failed to start." }
                val buffer = FloatArray(PcmAudio.FRAMES_PER_CHUNK * PcmAudio.CHANNEL_COUNT)
                while (!stop.get()) {
                    val read = openedStream.readFloat(buffer, 0, buffer.size, PcmAudio.IO_TIMEOUT_NANOS)
                    check(read >= 0) { "Oboe input read failed with code $read." }
                    if (read == 0) {
                        Thread.sleep(5L)
                        continue
                    }
                    val pcm = PcmAudio.floatToPcm16Bytes(buffer, read)
                    output.write(pcm)
                    dataBytes += pcm.size.toLong()
                    onProgress(read / PcmAudio.CHANNEL_COUNT, PcmAudio.audioLevel(buffer, read))
                }
                openedStream.requestStop()
            } finally {
                stream?.close()
                PcmAudio.writeWavHeader(output, dataBytes)
            }
        }
    }

    private fun openInputStream(): AudioStream {
        return AudioStreamBuilder()
            .setAudioApi(AudioApi.AAUDIO)
            .setDirection(AudioDirection.INPUT)
            .setSampleRate(PcmAudio.SAMPLE_RATE)
            .setChannelCount(PcmAudio.CHANNEL_COUNT)
            .setFormat(AudioFormat.FLOAT)
            .setPerformanceMode(PerformanceMode.LOW_LATENCY)
            .openStream()
    }
}
