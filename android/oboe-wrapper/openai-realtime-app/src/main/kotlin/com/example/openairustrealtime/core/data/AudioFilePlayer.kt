package com.example.openairustrealtime.core.data

import android.content.Context
import com.example.openairustrealtime.RealtimeNative
import java.io.File

class AudioFilePlayer(private val context: Context) {
    fun play(bytes: ByteArray, extension: String): File {
        val output = File(context.cacheDir, "openai-tts-output.$extension")
        output.writeBytes(bytes)
        val result = RealtimeNative.playPcmNative(bytes)
        if (result != 0) {
            throw IllegalStateException(
                RealtimeNative.nativeAudioErrorNative().orEmpty()
                    .ifBlank { "Native oboe TTS playback failed with code $result." }
            )
        }
        return output
    }

    fun stop() {
        RealtimeNative.stopNativeAudio()
    }
}
