package com.example.openairustrealtime.core.audio

data class RealtimeAudioDiagnostics(
    val inputXRunCount: Int = 0,
    val outputXRunCount: Int = 0,
    val inputBurstFrames: Int = 0,
    val outputBurstFrames: Int = 0,
    val inputBufferSizeFrames: Int = 0,
    val outputBufferSizeFrames: Int = 0,
    val inputBufferCapacityFrames: Int = 0,
    val outputBufferCapacityFrames: Int = 0,
    val outputLatencyMillis: Float = 0f,
    val outputPendingFrames: Long = 0L,
    val lastAsyncError: Int = 0
)
