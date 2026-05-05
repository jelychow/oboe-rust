package com.example.openairustrealtime.core.audio

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class RealtimeAudioEffectPolicyTest {
    @Test
    fun captureEffectsRequirePositiveSessionId() {
        assertTrue(RealtimeAudioEffectPolicy.canAttachToSession(1))
        assertFalse(RealtimeAudioEffectPolicy.canAttachToSession(0))
        assertFalse(RealtimeAudioEffectPolicy.canAttachToSession(-1))
    }

    @Test
    fun missingOboeSessionApiUsesNoSessionSentinel() {
        assertFalse(RealtimeAudioEffectPolicy.canAttachToSession(-1))
    }
}
