package com.example.openairustrealtime.core.network

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class RealtimeBargeInControllerTest {
    @Test
    fun respondingRequiresSustainedSpeechBeforeInterrupting() {
        val controller = RealtimeBargeInController(
            interruptLevelThreshold = 0.12f,
            requiredConsecutiveSpeechChunks = 3
        )

        assertFalse(controller.evaluate(status = "Responding", micLevel = 0.13f).shouldUploadMic)
        assertFalse(controller.evaluate(status = "Responding", micLevel = 0.14f).shouldUploadMic)

        val interrupt = controller.evaluate(status = "Responding", micLevel = 0.15f)

        assertTrue(interrupt.shouldUploadMic)
        assertTrue(interrupt.shouldCancelResponse)
    }

    @Test
    fun onceInterruptedKeepsUploadingWithoutRepeatedCancel() {
        val controller = RealtimeBargeInController(
            interruptLevelThreshold = 0.12f,
            requiredConsecutiveSpeechChunks = 1
        )

        val first = controller.evaluate(status = "Responding", micLevel = 0.2f)
        val second = controller.evaluate(status = "Responding", micLevel = 0.18f)

        assertTrue(first.shouldCancelResponse)
        assertTrue(second.shouldUploadMic)
        assertFalse(second.shouldCancelResponse)
    }

    @Test
    fun leavingRespondingStateResetsBargeInLatch() {
        val controller = RealtimeBargeInController(
            interruptLevelThreshold = 0.12f,
            requiredConsecutiveSpeechChunks = 1
        )

        controller.evaluate(status = "Responding", micLevel = 0.2f)
        val connected = controller.evaluate(status = "Connected", micLevel = 0.01f)
        val respondingAgain = controller.evaluate(status = "Responding", micLevel = 0.11f)

        assertTrue(connected.shouldUploadMic)
        assertFalse(respondingAgain.shouldUploadMic)
        assertFalse(respondingAgain.shouldCancelResponse)
    }
}
