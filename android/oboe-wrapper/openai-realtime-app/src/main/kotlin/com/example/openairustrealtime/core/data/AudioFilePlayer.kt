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
        },
        onFinished = {
            clearCachedOutput()
        }
    )
    @Volatile private var cachedOutput: File? = null

    @Synchronized
    fun play(bytes: ByteArray, extension: String): File {
        clearCachedOutput()
        val output = File(context.cacheDir, "openai-tts-output.$extension")
        output.writeBytes(bytes)
        cachedOutput = output
        stats.reset()
        player.playPcm16(bytes)
        return output
    }

    @Synchronized
    fun stop() {
        player.stop()
        clearCachedOutput()
    }

    @Synchronized
    private fun clearCachedOutput(): Boolean {
        val output = cachedOutput
        cachedOutput = null
        return CachedAudioFilePolicy.deleteIfPresent(output)
    }
}
