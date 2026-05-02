package com.example.openairustrealtime.core.data

import android.content.Context
import com.example.openairustrealtime.core.audio.OboeWavRecorder
import com.example.openairustrealtime.core.audio.PcmAudio
import java.io.File

class WavRecorder(
    private val context: Context,
    private val stats: AudioStatsTracker
) {
    private val recorder = OboeWavRecorder { frames, level ->
        stats.recordInput(frames, level)
    }
    private var currentFile: File? = null

    @Synchronized
    fun start(): File {
        check(currentFile == null) { "Recording is already running." }
        val file = File(context.cacheDir, "openai-asr-input.wav")
        stats.reset()
        recorder.start(file)
        currentFile = file
        return file
    }

    @Synchronized
    fun stop(): File {
        val file = currentFile ?: error("No recording is active.")
        recorder.stop()
        currentFile = null
        check(file.exists() && file.length() > PcmAudio.WAV_HEADER_BYTES) {
            "Oboe SDK recording did not produce microphone samples."
        }
        return file
    }

    @Synchronized
    fun cancel() {
        if (currentFile != null) {
            recorder.cancel()
            currentFile = null
        }
    }
}
