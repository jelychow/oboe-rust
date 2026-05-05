package com.example.openairustrealtime.core.audio

import java.io.RandomAccessFile
import kotlin.math.roundToInt
import kotlin.math.sqrt

object PcmAudio {
    const val SAMPLE_RATE = 24_000
    const val CHANNEL_COUNT = 1
    const val FRAMES_PER_CHUNK = 480
    const val CHUNK_DURATION_MILLIS = 20
    const val REALTIME_IO_TIMEOUT_NANOS = 20_000_000L
    const val IO_TIMEOUT_NANOS = 100_000_000L
    const val WAV_HEADER_BYTES = 44L

    fun pcm16ToFloatArray(pcm: ByteArray): FloatArray {
        val samples = FloatArray(pcm.size / 2)
        var input = 0
        for (index in samples.indices) {
            val value = ((pcm[input + 1].toInt() shl 8) or (pcm[input].toInt() and 0xff)).toShort()
            samples[index] = if (value < 0) value / 32768f else value / 32767f
            input += 2
        }
        return samples
    }

    fun floatToPcm16Bytes(audio: FloatArray, sampleCount: Int): ByteArray {
        val boundedCount = sampleCount.coerceIn(0, audio.size)
        val pcm = ByteArray(boundedCount * 2)
        var output = 0
        for (index in 0 until boundedCount) {
            val clipped = audio[index].coerceIn(-1f, 1f)
            val scaled = if (clipped < 0f) clipped * 32768f else clipped * 32767f
            val sample = scaled.roundToInt().toShort()
            pcm[output] = sample.toInt().toByte()
            pcm[output + 1] = (sample.toInt() ushr 8).toByte()
            output += 2
        }
        return pcm
    }

    fun audioLevel(audio: FloatArray, sampleCount: Int): Float {
        return audioLevel(audio, 0, sampleCount)
    }

    fun audioLevel(audio: FloatArray, offset: Int, sampleCount: Int): Float {
        val safeOffset = offset.coerceIn(0, audio.size)
        val boundedCount = sampleCount.coerceIn(0, audio.size)
        if (boundedCount == 0) return 0f
        var sumSquares = 0f
        val end = (safeOffset + boundedCount).coerceAtMost(audio.size)
        if (end <= safeOffset) return 0f
        for (index in safeOffset until end) {
            val clipped = audio[index].coerceIn(-1f, 1f)
            sumSquares += clipped * clipped
        }
        val rms = sqrt(sumSquares / (end - safeOffset))
        return (rms * 6f).coerceIn(0f, 1f)
    }

    fun audioLevelPcm16(pcm: ByteArray): Float {
        val audio = pcm16ToFloatArray(pcm)
        return audioLevel(audio, audio.size)
    }

    fun writeWavHeader(file: RandomAccessFile, dataBytes: Long) {
        val channels = CHANNEL_COUNT
        val bitsPerSample = 16
        val blockAlign = channels * (bitsPerSample / 8)
        val byteRate = SAMPLE_RATE * blockAlign
        file.seek(0L)
        file.writeAscii("RIFF")
        file.writeLittleEndianInt((36L + dataBytes).coerceAtMost(MAX_UINT32).toInt())
        file.writeAscii("WAVEfmt ")
        file.writeLittleEndianInt(16)
        file.writeLittleEndianShort(1)
        file.writeLittleEndianShort(channels)
        file.writeLittleEndianInt(SAMPLE_RATE)
        file.writeLittleEndianInt(byteRate)
        file.writeLittleEndianShort(blockAlign)
        file.writeLittleEndianShort(bitsPerSample)
        file.writeAscii("data")
        file.writeLittleEndianInt(dataBytes.coerceAtMost(MAX_UINT32).toInt())
    }

    private const val MAX_UINT32 = 4_294_967_295L

    private fun RandomAccessFile.writeAscii(value: String) {
        write(value.toByteArray(Charsets.US_ASCII))
    }

    private fun RandomAccessFile.writeLittleEndianInt(value: Int) {
        write(byteArrayOf(
            value.toByte(),
            (value ushr 8).toByte(),
            (value ushr 16).toByte(),
            (value ushr 24).toByte()
        ))
    }

    private fun RandomAccessFile.writeLittleEndianShort(value: Int) {
        write(byteArrayOf(value.toByte(), (value ushr 8).toByte()))
    }
}
