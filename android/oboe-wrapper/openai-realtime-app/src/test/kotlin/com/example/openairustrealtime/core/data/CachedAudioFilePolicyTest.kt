package com.example.openairustrealtime.core.data

import java.io.File
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class CachedAudioFilePolicyTest {
    @Test
    fun deletesExistingCacheFile() {
        val file = File.createTempFile("cached-audio", ".pcm").apply {
            writeText("audio")
        }

        assertTrue(CachedAudioFilePolicy.deleteIfPresent(file))
        assertFalse(file.exists())
    }

    @Test
    fun ignoresMissingCacheFile() {
        val file = File.createTempFile("cached-audio-missing", ".wav")
        file.delete()

        assertFalse(CachedAudioFilePolicy.deleteIfPresent(file))
        assertFalse(file.exists())
    }
}
