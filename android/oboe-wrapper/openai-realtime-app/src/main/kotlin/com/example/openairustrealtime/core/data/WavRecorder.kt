package com.example.openairustrealtime.core.data

import android.content.Context
import com.example.openairustrealtime.RealtimeNative
import java.io.File

class WavRecorder(private val context: Context) {
    private var currentFile: File? = null

    @Synchronized
    fun start(): File {
        check(currentFile == null) { "Recording is already running." }
        val file = File(context.cacheDir, "openai-asr-input.wav")
        val result = RealtimeNative.startWavRecordingNative(file.absolutePath)
        if (result != 0) {
            throw IllegalStateException(nativeAudioError(result, "Native oboe ASR recording failed to start"))
        }
        currentFile = file
        return file
    }

    @Synchronized
    fun stop(): File {
        val file = currentFile ?: error("No recording is active.")
        val result = RealtimeNative.stopWavRecordingNative()
        currentFile = null
        if (result != 0) {
            throw IllegalStateException(nativeAudioError(result, "Native oboe ASR recording failed to stop"))
        }
        check(file.exists() && file.length() > WAV_HEADER_BYTES) {
            "Native oboe ASR recording did not produce microphone samples."
        }
        return file
    }

    @Synchronized
    fun cancel() {
        if (currentFile != null) {
            RealtimeNative.stopWavRecordingNative()
            currentFile = null
        }
    }

    private fun nativeAudioError(code: Int, fallback: String): String {
        return RealtimeNative.nativeAudioErrorNative().orEmpty()
            .ifBlank { "$fallback with code $code." }
    }

    companion object {
        private const val WAV_HEADER_BYTES = 44L
    }
}
