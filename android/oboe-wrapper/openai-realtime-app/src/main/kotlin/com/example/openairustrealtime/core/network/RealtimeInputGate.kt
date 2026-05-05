package com.example.openairustrealtime.core.network

internal object RealtimeInputGate {
    fun shouldUploadMic(status: String): Boolean {
        return status != "Responding"
    }
}
