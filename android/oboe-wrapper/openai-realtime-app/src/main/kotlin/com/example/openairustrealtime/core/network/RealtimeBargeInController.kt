package com.example.openairustrealtime.core.network

internal data class RealtimeBargeInDecision(
    val shouldUploadMic: Boolean,
    val shouldCancelResponse: Boolean
)

internal class RealtimeBargeInController(
    private val interruptLevelThreshold: Float = 0.12f,
    private val requiredConsecutiveSpeechChunks: Int = 4
) {
    private var consecutiveSpeechChunks = 0
    private var interruptedCurrentResponse = false

    fun evaluate(status: String, micLevel: Float): RealtimeBargeInDecision {
        if (status != STATUS_RESPONDING && status != STATUS_INTERRUPTING) {
            reset()
            return RealtimeBargeInDecision(shouldUploadMic = true, shouldCancelResponse = false)
        }
        if (status == STATUS_INTERRUPTING || interruptedCurrentResponse) {
            return RealtimeBargeInDecision(shouldUploadMic = true, shouldCancelResponse = false)
        }
        if (micLevel >= interruptLevelThreshold) {
            consecutiveSpeechChunks += 1
        } else {
            consecutiveSpeechChunks = 0
        }
        if (consecutiveSpeechChunks >= requiredConsecutiveSpeechChunks) {
            interruptedCurrentResponse = true
            return RealtimeBargeInDecision(shouldUploadMic = true, shouldCancelResponse = true)
        }
        return RealtimeBargeInDecision(shouldUploadMic = false, shouldCancelResponse = false)
    }

    fun onStatusChanged(status: String) {
        if (status != STATUS_RESPONDING && status != STATUS_INTERRUPTING) {
            reset()
        }
    }

    private fun reset() {
        consecutiveSpeechChunks = 0
        interruptedCurrentResponse = false
    }

    private companion object {
        private const val STATUS_RESPONDING = "Responding"
        private const val STATUS_INTERRUPTING = "Interrupting"
    }
}
