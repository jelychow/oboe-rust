package com.example.openairustrealtime.core.network

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class RealtimeAssistantPlaybackControllerTest {
    @Test
    fun requestInterrupt_abortsCurrentPlaybackAndDropsStaleAudioUntilResponseEnds() {
        val controller = RealtimeAssistantPlaybackController()

        controller.requestInterrupt()

        assertTrue(controller.shouldAbortPlayback())
        assertTrue(controller.shouldDropIncomingAudio())

        controller.onStatusChanged("Interrupting")
        assertTrue(controller.shouldAbortPlayback())
        assertTrue(controller.shouldDropIncomingAudio())

        controller.onStatusChanged("Connected")
        assertFalse(controller.shouldAbortPlayback())
        assertFalse(controller.shouldDropIncomingAudio())
    }

    @Test
    fun responseStartedClearsPreviousInterruptLatch() {
        val controller = RealtimeAssistantPlaybackController()

        controller.requestInterrupt()
        controller.onResponseStarted()

        assertFalse(controller.shouldAbortPlayback())
        assertFalse(controller.shouldDropIncomingAudio())
    }

    @Test
    fun nonInterruptStatusesKeepPlaybackEnabled() {
        val controller = RealtimeAssistantPlaybackController()

        controller.onStatusChanged("Responding")

        assertFalse(controller.shouldAbortPlayback())
        assertFalse(controller.shouldDropIncomingAudio())
    }
}
