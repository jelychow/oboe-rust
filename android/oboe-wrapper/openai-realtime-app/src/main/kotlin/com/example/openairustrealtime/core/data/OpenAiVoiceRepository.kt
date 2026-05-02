package com.example.openairustrealtime.core.data

import android.content.Context
import com.example.openairustrealtime.RealtimeNative
import com.example.openairustrealtime.core.model.SpeechRequest
import com.example.openairustrealtime.core.model.TranscriptionRequest
import com.example.openairustrealtime.core.network.OpenAiAudioApi
import java.io.File

class OpenAiVoiceRepository(context: Context) {
    private val appContext = context.applicationContext
    private val api = OpenAiAudioApi()
    private val player = AudioFilePlayer(appContext)
    private val recorder = WavRecorder(appContext)
    val apiKeyStore = ApiKeyStore(appContext)

    fun synthesizeAndPlay(apiKey: String, request: SpeechRequest): File {
        val bytes = api.synthesizeSpeech(apiKey, request)
        return player.play(bytes, request.responseFormat)
    }

    fun startRecording(): File = recorder.start()

    fun stopRecordingAndTranscribe(apiKey: String, request: TranscriptionRequest): String {
        val file = recorder.stop()
        return api.transcribe(apiKey, request, file)
    }

    fun cancelRecording() {
        recorder.cancel()
    }

    fun startRealtime(apiKey: String, model: String, instructions: String): Int {
        return RealtimeNative.startNative(apiKey, model, instructions)
    }

    fun stopRealtime(): Int = RealtimeNative.stopNative()

    fun realtimeStatus(): String = RealtimeNative.statusNative().orEmpty()

    fun realtimeTranscript(): String = RealtimeNative.transcriptNative().orEmpty()

    fun realtimeError(): String = RealtimeNative.lastErrorNative().orEmpty()

    fun realtimeStats(): String = RealtimeNative.statsNative().orEmpty()

    fun close() {
        recorder.cancel()
        player.stop()
        RealtimeNative.stopNative()
    }
}
