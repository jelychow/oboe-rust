package com.example.openairustrealtime.core.audio

import com.google.oboe.AudioApi
import com.google.oboe.AudioDirection
import com.google.oboe.AudioFormat
import com.google.oboe.PerformanceMode
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class RealtimeAudioStreamOpenPlanTest {
    @Test
    fun inputPlanStartsWithVoiceSessionAndFallsBackToGenericPcm() {
        val specs = RealtimeAudioStreamOpenPlan.specsFor(AudioDirection.INPUT)

        assertEquals(
            listOf(
                "aaudio-low-latency-float-voice-session",
                "aaudio-low-latency-float-voice",
                "aaudio-low-latency-float-generic",
                "aaudio-balanced-float-generic",
                "aaudio-balanced-i16-generic"
            ),
            specs.map { it.label }
        )
        assertTrue(specs.first().useVoiceCommunicationInput)
        assertTrue(specs.first().allocateInputSession)
        assertFalse(specs[1].allocateInputSession)
        assertFalse(specs.last().useVoiceCommunicationInput)
        assertEquals(AudioFormat.I16, specs.last().format)
        assertEquals(PerformanceMode.NONE, specs.last().performanceMode)
        assertTrue(specs.all { it.sampleRate == PcmAudio.SAMPLE_RATE })
        assertTrue(specs.all { it.channelCount == PcmAudio.CHANNEL_COUNT })
    }

    @Test
    fun outputPlanKeepsAaudioFirstAndFallsBackToOpenSlPcm() {
        val specs = RealtimeAudioStreamOpenPlan.specsFor(AudioDirection.OUTPUT)

        assertEquals(
            listOf(
                "aaudio-low-latency-float",
                "aaudio-balanced-float",
                "aaudio-balanced-i16",
                "opensles-balanced-i16"
            ),
            specs.map { it.label }
        )
        assertEquals(AudioApi.AAUDIO, specs.first().audioApi)
        assertEquals(AudioFormat.FLOAT, specs.first().format)
        assertEquals(PerformanceMode.LOW_LATENCY, specs.first().performanceMode)
        assertEquals(AudioApi.OPENSL_ES, specs.last().audioApi)
        assertEquals(AudioFormat.I16, specs.last().format)
        assertFalse(specs.any { it.useVoiceCommunicationInput })
        assertFalse(specs.any { it.allocateInputSession })
        assertTrue(specs.take(3).all { it.useVoiceCommunicationOutput })
        assertFalse(specs.last().useVoiceCommunicationOutput)
        assertTrue(specs.all { it.sampleRate == PcmAudio.SAMPLE_RATE })
        assertTrue(specs.all { it.channelCount == PcmAudio.CHANNEL_COUNT })
    }
}
