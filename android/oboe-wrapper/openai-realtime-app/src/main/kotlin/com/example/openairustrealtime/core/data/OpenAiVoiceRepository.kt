package com.example.openairustrealtime.core.data

import android.content.Context
import android.util.Log
import com.example.openairustrealtime.core.audio.RealtimeOboeAudioPump
import com.example.openairustrealtime.core.model.SpeechRequest
import com.example.openairustrealtime.core.model.TranscriptionRequest
import com.example.openairustrealtime.core.network.OpenAiAudioApi
import com.example.openairustrealtime.core.network.OpenAiRealtimeSession
import com.example.openairustrealtime.core.util.AppLog
import java.io.File

class OpenAiVoiceRepository(context: Context) {
    private val appContext = context.applicationContext
    private val api = OpenAiAudioApi()
    private val audioStats = AudioStatsTracker()
    private val player = AudioFilePlayer(appContext, audioStats)
    private val recorder = WavRecorder(appContext, audioStats)
    private val realtime = OpenAiRealtimeSession()
    private val realtimeAudioPump = RealtimeOboeAudioPump(
        context = appContext,
        onInputAudio = { audio, sampleCount -> realtime.appendInputAudio(audio, sampleCount) },
        pollOutputAudio = realtime::pollOutputAudio,
        shouldInterruptOutput = realtime::shouldInterruptOutput,
        onOutputAudio = realtime::recordOutputAudio,
        onDiagnostics = realtime::recordAudioDiagnostics,
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
        return try {
            api.transcribe(apiKey, request, file)
        } finally {
            CachedAudioFilePolicy.deleteIfPresent(file)
        }
    }

    fun cancelRecording() {
        recorder.cancel()
    }

    fun startRealtime(apiKey: String, model: String, instructions: String): Int {
        AppLog.i(TAG, "Repository starting realtime model=$model")
        statsSource = StatsSource.REALTIME
        val result = realtime.start(apiKey, model, instructions)
        if (result == 0) {
            AppLog.i(TAG, "Realtime session ready; starting Oboe audio pump")
            realtimeAudioPump.start()
        } else {
            Log.w(TAG, "Realtime session start returned $result; audio pump not started")
        }
        return result
    }

    fun stopRealtime(reason: String = "unspecified"): Int {
        AppLog.i(TAG, "Repository stopping realtime reason=$reason")
        realtimeAudioPump.stop(reason)
        return realtime.stop(reason)
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
        AppLog.d(TAG, "Repository close")
        recorder.cancel()
        player.stop()
        realtimeAudioPump.stop("repository.close")
        realtime.close()
    }

    private enum class StatsSource {
        GENERAL,
        REALTIME
    }

    private companion object {
        private const val TAG = "OpenAiVoiceRepository"
    }
}
