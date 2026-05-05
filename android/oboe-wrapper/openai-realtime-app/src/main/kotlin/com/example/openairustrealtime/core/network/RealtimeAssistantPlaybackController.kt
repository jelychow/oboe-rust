package com.example.openairustrealtime.core.network

internal class RealtimeAssistantPlaybackController {
    @Volatile private var interruptRequested = false

    fun requestInterrupt() {
        interruptRequested = true
    }

    fun onResponseStarted() {
        interruptRequested = false
    }

    fun onStatusChanged(status: String) {
        if (status != STATUS_RESPONDING && status != STATUS_INTERRUPTING) {
            interruptRequested = false
        }
    }

    fun shouldDropIncomingAudio(): Boolean = interruptRequested

    fun shouldAbortPlayback(): Boolean = interruptRequested

    private companion object {
        private const val STATUS_RESPONDING = "Responding"
        private const val STATUS_INTERRUPTING = "Interrupting"
    }
}
