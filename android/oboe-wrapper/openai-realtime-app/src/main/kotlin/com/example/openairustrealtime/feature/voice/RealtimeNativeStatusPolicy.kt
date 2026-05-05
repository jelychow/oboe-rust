package com.example.openairustrealtime.feature.voice

import com.example.openairustrealtime.core.model.VoiceUiState

internal data class RealtimeNativeStatusDecision(
    val effectiveRawStatus: String,
    val status: String,
    val shouldCleanupNativeSession: Boolean,
    val ignoredStartingStopped: Boolean
)

internal object RealtimeNativeStatusPolicy {
    fun evaluate(rawStatus: String, state: VoiceUiState): RealtimeNativeStatusDecision {
        val normalizedRaw = rawStatus.trim()
        val starting = state.realtimeRunning && state.busy && state.status == "Connecting"
        val ignoreStoppedDuringStart = starting && normalizedRaw == "Stopped"
        val effectiveRaw = if (ignoreStoppedDuringStart) "" else normalizedRaw
        val status = effectiveRaw.ifBlank {
            if (state.realtimeRunning) state.status.ifBlank { "Connecting" } else "Stopped"
        }
        return RealtimeNativeStatusDecision(
            effectiveRawStatus = effectiveRaw,
            status = status,
            shouldCleanupNativeSession = state.realtimeRunning &&
                !ignoreStoppedDuringStart &&
                (effectiveRaw == "Stopped" || status == "Error"),
            ignoredStartingStopped = ignoreStoppedDuringStart
        )
    }
}
