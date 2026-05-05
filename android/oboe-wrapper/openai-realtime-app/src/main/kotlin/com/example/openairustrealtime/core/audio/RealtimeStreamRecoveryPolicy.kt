package com.example.openairustrealtime.core.audio

internal class RealtimeStreamRecoveryPolicy(
    private val maxRecoveryAttempts: Int
) {
    private var consecutiveRecoverableFailures = 0

    fun recordStreamOpened() {
        // Opening a stream is not enough to prove the audio path recovered.
    }

    fun recordSuccessfulAudioIo() {
        consecutiveRecoverableFailures = 0
    }

    fun recordRecoverableFailure(): Boolean {
        consecutiveRecoverableFailures += 1
        return consecutiveRecoverableFailures <= maxRecoveryAttempts
    }
}

internal class RealtimeZeroProgressPolicy(
    private val maxZeroProgressWrites: Int
) {
    private var consecutiveZeroProgressWrites = 0

    fun recordSuccessfulWrite() {
        consecutiveZeroProgressWrites = 0
    }

    fun recordZeroProgressWrite(): Boolean {
        consecutiveZeroProgressWrites += 1
        return consecutiveZeroProgressWrites < maxZeroProgressWrites
    }
}
