package com.example.openairustrealtime.core.model

data class VoiceUiState(
    val selectedMode: VoiceMode = VoiceMode.REALTIME_CHAT,
    val status: String = "Ready",
    val statusDetail: String = "Choose a voice workflow.",
    val savedKeyPresent: Boolean = false,
    val micPermissionGranted: Boolean = false,
    val busy: Boolean = false,
    val recording: Boolean = false,
    val realtimeRunning: Boolean = false,
    val resultTitle: String = "Realtime output",
    val resultText: String = "Transcripts, generated audio status, and translated responses will appear here.",
    val lastError: String = "",
    val events: List<String> = listOf("Ready"),
    val stats: RealtimeStats = RealtimeStats(),
    val micLevel: Float = 0f,
    val outputLevel: Float = 0f
)
