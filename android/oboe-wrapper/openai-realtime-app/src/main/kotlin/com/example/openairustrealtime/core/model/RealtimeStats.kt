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
    val outputLevel: Float = 0f
) {
    companion object {
        private val statsPattern: Pattern = Pattern.compile(
            "Mic sent: (\\d+) chunks / (\\d+) frames\\. Mic dropped: (\\d+) chunks / (\\d+) frames\\. Output played: (\\d+) chunks / (\\d+) frames\\.(?: Levels: mic ([0-9.]+), output ([0-9.]+)\\.)?"
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
                micLevel = matcher.group(7).toSafeFloat(),
                outputLevel = matcher.group(8).toSafeFloat()
            )
        }

        private fun String?.toSafeLong(): Long = this?.toLongOrNull() ?: 0L

        private fun String?.toSafeFloat(): Float = this?.toFloatOrNull()?.coerceIn(0f, 1f) ?: 0f
    }
}
