package com.example.openairustrealtime.core.data

import com.example.openairustrealtime.core.util.SecretRedactor
import java.util.Locale

class AudioStatsTracker {
    private var inputChunks = 0L
    private var inputFrames = 0L
    private var outputChunks = 0L
    private var outputFrames = 0L
    private var inputLevel = 0f
    private var outputLevel = 0f
    private var lastError = ""

    @Synchronized
    fun reset() {
        inputChunks = 0L
        inputFrames = 0L
        outputChunks = 0L
        outputFrames = 0L
        inputLevel = 0f
        outputLevel = 0f
        lastError = ""
    }

    @Synchronized
    fun recordInput(frames: Int, level: Float) {
        inputChunks = inputChunks.saturatingInc()
        inputFrames = inputFrames.saturatingAdd(frames.coerceAtLeast(0).toLong())
        inputLevel = level.coerceIn(0f, 1f)
    }

    @Synchronized
    fun recordOutput(frames: Int, level: Float) {
        outputChunks = outputChunks.saturatingInc()
        outputFrames = outputFrames.saturatingAdd(frames.coerceAtLeast(0).toLong())
        outputLevel = level.coerceIn(0f, 1f)
    }

    @Synchronized
    fun reportError(error: String) {
        lastError = SecretRedactor.redact(error)
    }

    @Synchronized
    fun lastError(): String = lastError

    @Synchronized
    fun stats(): String {
        return "Mic sent: $inputChunks chunks / $inputFrames frames. " +
            "Mic dropped: 0 chunks / 0 frames. " +
            "Output played: $outputChunks chunks / $outputFrames frames. " +
            "Levels: mic ${String.format(Locale.US, "%.3f", inputLevel)}, " +
            "output ${String.format(Locale.US, "%.3f", outputLevel)}."
    }

    private fun Long.saturatingInc(): Long = if (this == Long.MAX_VALUE) Long.MAX_VALUE else this + 1L

    private fun Long.saturatingAdd(value: Long): Long {
        if (value <= 0L) return this
        return if (Long.MAX_VALUE - this < value) Long.MAX_VALUE else this + value
    }
}
