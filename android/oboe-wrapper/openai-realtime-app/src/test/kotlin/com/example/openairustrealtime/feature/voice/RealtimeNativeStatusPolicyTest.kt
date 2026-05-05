package com.example.openairustrealtime.feature.voice

import com.example.openairustrealtime.core.model.VoiceUiState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class RealtimeNativeStatusPolicyTest {
    @Test
    fun ignoresStoppedWhileRealtimeStartIsStillConnecting() {
        val decision = RealtimeNativeStatusPolicy.evaluate(
            rawStatus = "Stopped",
            state = VoiceUiState(
                status = "Connecting",
                busy = true,
                realtimeRunning = true
            )
        )

        assertEquals("", decision.effectiveRawStatus)
        assertEquals("Connecting", decision.status)
        assertTrue(decision.ignoredStartingStopped)
        assertFalse(decision.shouldCleanupNativeSession)
    }

    @Test
    fun cleansUpStoppedAfterRealtimeIsNoLongerStarting() {
        val decision = RealtimeNativeStatusPolicy.evaluate(
            rawStatus = "Stopped",
            state = VoiceUiState(
                status = "Connected",
                busy = false,
                realtimeRunning = true
            )
        )

        assertEquals("Stopped", decision.effectiveRawStatus)
        assertEquals("Stopped", decision.status)
        assertFalse(decision.ignoredStartingStopped)
        assertTrue(decision.shouldCleanupNativeSession)
    }
}
