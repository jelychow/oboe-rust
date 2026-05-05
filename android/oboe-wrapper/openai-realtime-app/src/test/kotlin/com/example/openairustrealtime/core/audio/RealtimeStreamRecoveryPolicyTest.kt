package com.example.openairustrealtime.core.audio

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class RealtimeStreamRecoveryPolicyTest {
    @Test
    fun streamOpenDoesNotResetConsecutiveRecoverableFailures() {
        val policy = RealtimeStreamRecoveryPolicy(maxRecoveryAttempts = 2)

        assertTrue(policy.recordRecoverableFailure())
        policy.recordStreamOpened()
        assertTrue(policy.recordRecoverableFailure())
        policy.recordStreamOpened()
        assertFalse(policy.recordRecoverableFailure())
    }

    @Test
    fun successfulAudioIoResetsConsecutiveRecoverableFailures() {
        val policy = RealtimeStreamRecoveryPolicy(maxRecoveryAttempts = 2)

        assertTrue(policy.recordRecoverableFailure())
        policy.recordSuccessfulAudioIo()
        assertTrue(policy.recordRecoverableFailure())
        assertTrue(policy.recordRecoverableFailure())
        assertFalse(policy.recordRecoverableFailure())
    }

    @Test
    fun zeroProgressPolicyEscalatesAfterConsecutiveTimeouts() {
        val policy = RealtimeZeroProgressPolicy(maxZeroProgressWrites = 2)

        assertTrue(policy.recordZeroProgressWrite())
        assertFalse(policy.recordZeroProgressWrite())
    }
}
