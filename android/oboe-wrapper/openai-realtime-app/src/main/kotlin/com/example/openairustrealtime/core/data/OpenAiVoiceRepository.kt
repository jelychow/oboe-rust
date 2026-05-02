package com.example.openairustrealtime.core.data

import android.content.Context
import com.example.openairustrealtime.core.audio.RealtimeOboeAudioPump
import com.example.openairustrealtime.core.model.SpeechRequest
import com.example.openairustrealtime.core.model.TranscriptionRequest
import com.example.openairustrealtime.core.network.OpenAiAudioApi
import com.example.openairustrealtime.core.network.OpenAiRealtimeSession
import java.io.File

class OpenAiVoiceRepository(context: Context) {
    private val appContext = context.applicationContext
    private val api = OpenAiAudioApi()
    private val audioStats = AudioStatsTracker()
    private val player = AudioFilePlayer(appContext, audioStats)
    private val recorder = WavRecorder(appContext, audioStats)
    private val realtime = OpenAiRealtimeSession()
    private val realtimeAudioPump = RealtimeOboeAudioPump(
        onInputAudio = { audio, sampleCount -> realtime.appendInputAudio(audio, sampleCount) },
        pollOutputAudio = realtime::pollOutputAudio,
        onOutputAudio = realtime::recordOutputAudio,
        onError = realtime::reportAudioError
    )
    @Volatile private var statsSource = StatsSource.GENERAL
    val apiKeyStore = ApiKeyStore(appContext)

    fun synthesizeAndPlay(apiKey: String, request: SpeechRequest): File {
        statsSource = StatsSource.GENERAL
        val bytes = api.synthesizeSpeech(apiKey, request)
        return player.play(bytes, request.responseFormat)
    }

    fun startRecording(): File {
        statsSource = StatsSource.GENERAL
        return recorder.start()
    }

    fun stopRecordingAndTranscribe(apiKey: String, request: TranscriptionRequest): String {
        val file = recorder.stop()
        return api.transcribe(apiKey, request, file)
    }

    fun cancelRecording() {
        recorder.cancel()
    }

    fun startRealtime(apiKey: String, model: String, instructions: String): Int {
        statsSource = StatsSource.REALTIME
        val result = realtime.start(apiKey, model, instructions)
        if (result == 0) {
            realtimeAudioPump.start()
        }
        return result
    }

    fun stopRealtime(): Int {
        realtimeAudioPump.stop()
        return realtime.stop()
    }

    fun realtimeStatus(): String = realtime.status()

    fun realtimeTranscript(): String = realtime.transcript()

    fun realtimeError(): String = when (statsSource) {
        StatsSource.REALTIME -> realtime.lastError()
        StatsSource.GENERAL -> audioStats.lastError()
    }

    fun realtimeStats(): String = when (statsSource) {
        StatsSource.REALTIME -> realtime.stats()
        StatsSource.GENERAL -> audioStats.stats()
    }

    fun close() {
        recorder.cancel()
        player.stop()
        realtimeAudioPump.stop()
        realtime.close()
    }

    private enum class StatsSource {
        GENERAL,
        REALTIME
    }
}
