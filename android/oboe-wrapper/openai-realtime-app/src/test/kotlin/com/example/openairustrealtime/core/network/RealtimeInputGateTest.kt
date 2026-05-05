package com.example.openairustrealtime.core.network

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class RealtimeInputGateTest {
    @Test
    fun blocksMicWhileAssistantIsResponding() {
        assertFalse(RealtimeInputGate.shouldUploadMic("Responding"))
    }

    @Test
    fun allowsMicForListeningAndConnectedStates() {
        assertTrue(RealtimeInputGate.shouldUploadMic("Listening"))
        assertTrue(RealtimeInputGate.shouldUploadMic("Connected"))
        assertTrue(RealtimeInputGate.shouldUploadMic("Thinking"))
        assertTrue(RealtimeInputGate.shouldUploadMic(""))
    }
}
