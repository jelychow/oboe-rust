package com.example.openairustrealtime.core.audio

import com.google.oboe.AudioApi
import com.google.oboe.AudioDirection
import com.google.oboe.AudioFormat
import com.google.oboe.PerformanceMode

internal data class RealtimeAudioStreamOpenSpec(
    val label: String,
    val direction: AudioDirection,
    val audioApi: AudioApi,
    val sampleRate: Int = PcmAudio.SAMPLE_RATE,
    val channelCount: Int = PcmAudio.CHANNEL_COUNT,
    val format: AudioFormat,
    val performanceMode: PerformanceMode,
    val useVoiceCommunicationInput: Boolean = false,
    val allocateInputSession: Boolean = false,
    val useVoiceCommunicationOutput: Boolean = false
)

internal object RealtimeAudioStreamOpenPlan {
    fun specsFor(direction: AudioDirection): List<RealtimeAudioStreamOpenSpec> {
        return if (direction == AudioDirection.INPUT) inputSpecs else outputSpecs
    }

    private val inputSpecs = listOf(
        RealtimeAudioStreamOpenSpec(
            label = "aaudio-low-latency-float-voice-session",
            direction = AudioDirection.INPUT,
            audioApi = AudioApi.AAUDIO,
            format = AudioFormat.FLOAT,
            performanceMode = PerformanceMode.LOW_LATENCY,
            useVoiceCommunicationInput = true,
            allocateInputSession = true
        ),
        RealtimeAudioStreamOpenSpec(
            label = "aaudio-low-latency-float-voice",
            direction = AudioDirection.INPUT,
            audioApi = AudioApi.AAUDIO,
            format = AudioFormat.FLOAT,
            performanceMode = PerformanceMode.LOW_LATENCY,
            useVoiceCommunicationInput = true
        ),
        RealtimeAudioStreamOpenSpec(
            label = "aaudio-low-latency-float-generic",
            direction = AudioDirection.INPUT,
            audioApi = AudioApi.AAUDIO,
            format = AudioFormat.FLOAT,
            performanceMode = PerformanceMode.LOW_LATENCY
        ),
        RealtimeAudioStreamOpenSpec(
            label = "aaudio-balanced-float-generic",
            direction = AudioDirection.INPUT,
            audioApi = AudioApi.AAUDIO,
            format = AudioFormat.FLOAT,
            performanceMode = PerformanceMode.NONE
        ),
        RealtimeAudioStreamOpenSpec(
            label = "aaudio-balanced-i16-generic",
            direction = AudioDirection.INPUT,
            audioApi = AudioApi.AAUDIO,
            format = AudioFormat.I16,
            performanceMode = PerformanceMode.NONE
        )
    )

    private val outputSpecs = listOf(
        RealtimeAudioStreamOpenSpec(
            label = "aaudio-low-latency-float",
            direction = AudioDirection.OUTPUT,
            audioApi = AudioApi.AAUDIO,
            format = AudioFormat.FLOAT,
            performanceMode = PerformanceMode.LOW_LATENCY,
            useVoiceCommunicationOutput = true
        ),
        RealtimeAudioStreamOpenSpec(
            label = "aaudio-balanced-float",
            direction = AudioDirection.OUTPUT,
            audioApi = AudioApi.AAUDIO,
            format = AudioFormat.FLOAT,
            performanceMode = PerformanceMode.NONE,
            useVoiceCommunicationOutput = true
        ),
        RealtimeAudioStreamOpenSpec(
            label = "aaudio-balanced-i16",
            direction = AudioDirection.OUTPUT,
            audioApi = AudioApi.AAUDIO,
            format = AudioFormat.I16,
            performanceMode = PerformanceMode.NONE,
            useVoiceCommunicationOutput = true
        ),
        RealtimeAudioStreamOpenSpec(
            label = "opensles-balanced-i16",
            direction = AudioDirection.OUTPUT,
            audioApi = AudioApi.OPENSL_ES,
            format = AudioFormat.I16,
            performanceMode = PerformanceMode.NONE
        )
    )
}
