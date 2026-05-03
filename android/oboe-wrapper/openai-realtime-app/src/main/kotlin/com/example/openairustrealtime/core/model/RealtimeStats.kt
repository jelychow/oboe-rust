package com.example.openairustrealtime.core.model

import java.util.regex.Pattern

data class RealtimeStats(
    val inputChunks: Long = 0L,
    val inputFrames: Long = 0L,
    val droppedInputChunks: Long = 0L,
    val droppedInputFrames: Long = 0L,
    val outputChunks: Long = 0L,
    val outputFrames: Long = 0L,
    val micLevel: Float = 0f,
    val outputLevel: Float = 0f,
    val inputXRunCount: Int = 0,
    val outputXRunCount: Int = 0,
    val outputLatencyMillis: Float = 0f,
    val outputPendingFrames: Long = 0L,
    val inputBufferSizeFrames: Int = 0,
    val inputBufferCapacityFrames: Int = 0,
    val inputBurstFrames: Int = 0,
    val outputBufferSizeFrames: Int = 0,
    val outputBufferCapacityFrames: Int = 0,
    val outputBurstFrames: Int = 0,
    val lastAsyncError: Int = 0
) {
    val totalXRunCount: Int
        get() = inputXRunCount + outputXRunCount

    companion object {
        private val statsPattern: Pattern = Pattern.compile(
            "Mic sent: (\\d+) chunks / (\\d+) frames\\. " +
                "Mic dropped: (\\d+) chunks / (\\d+) frames\\. " +
                "Output played: (\\d+) chunks / (\\d+) frames\\." +
                "(?: Levels: mic ([0-9.]+), output ([0-9.]+)\\.)?" +
                "(?: Diagnostics: xruns input (\\d+) / output (\\d+)\\. " +
                "Output latency ([0-9]+(?:\\.[0-9]+)?) ms / (\\d+) frames pending\\. " +
                "Buffer: input (\\d+)/(\\d+) burst (\\d+), output (\\d+)/(\\d+) burst (\\d+)\\. " +
                "Async error: (-?\\d+)\\.)?"
        )

        fun parse(text: String?): RealtimeStats {
            val matcher = statsPattern.matcher(text.orEmpty())
            if (!matcher.matches()) return RealtimeStats()
            return RealtimeStats(
                inputChunks = matcher.group(1).toSafeLong(),
                inputFrames = matcher.group(2).toSafeLong(),
                droppedInputChunks = matcher.group(3).toSafeLong(),
                droppedInputFrames = matcher.group(4).toSafeLong(),
                outputChunks = matcher.group(5).toSafeLong(),
                outputFrames = matcher.group(6).toSafeLong(),
                micLevel = matcher.group(7).toSafeLevelFloat(),
                outputLevel = matcher.group(8).toSafeLevelFloat(),
                inputXRunCount = matcher.group(9).toSafeInt(),
                outputXRunCount = matcher.group(10).toSafeInt(),
                outputLatencyMillis = matcher.group(11).toSafeFloat(),
                outputPendingFrames = matcher.group(12).toSafeLong(),
                inputBufferSizeFrames = matcher.group(13).toSafeInt(),
                inputBufferCapacityFrames = matcher.group(14).toSafeInt(),
                inputBurstFrames = matcher.group(15).toSafeInt(),
                outputBufferSizeFrames = matcher.group(16).toSafeInt(),
                outputBufferCapacityFrames = matcher.group(17).toSafeInt(),
                outputBurstFrames = matcher.group(18).toSafeInt(),
                lastAsyncError = matcher.group(19).toSafeInt()
            )
        }

        private fun String?.toSafeLong(): Long = this?.toLongOrNull() ?: 0L

        private fun String?.toSafeInt(): Int = this?.toIntOrNull() ?: 0

        private fun String?.toSafeFloat(): Float = this?.toFloatOrNull() ?: 0f

        private fun String?.toSafeLevelFloat(): Float = toSafeFloat().coerceIn(0f, 1f)
    }
}
