package com.example.openairustrealtime.core.data

import android.content.Context
import com.example.openairustrealtime.core.audio.OboePcmPlayer
import java.io.File

class AudioFilePlayer(
    private val context: Context,
    private val stats: AudioStatsTracker
) {
    private val player = OboePcmPlayer(
        onProgress = { frames, level ->
            stats.recordOutput(frames, level)
        },
        onError = { error ->
            stats.reportError(error)
        }
    )

    fun play(bytes: ByteArray, extension: String): File {
        val output = File(context.cacheDir, "openai-tts-output.$extension")
        output.writeBytes(bytes)
        stats.reset()
        player.playPcm16(bytes)
        return output
    }

    fun stop() {
        player.stop()
    }
}
