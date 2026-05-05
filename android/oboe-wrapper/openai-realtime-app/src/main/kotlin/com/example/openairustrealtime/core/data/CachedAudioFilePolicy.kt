package com.example.openairustrealtime.core.data

import java.io.File

object CachedAudioFilePolicy {
    fun deleteIfPresent(file: File?): Boolean {
        if (file == null || !file.exists()) return false
        return runCatching { file.delete() }.getOrDefault(false)
    }
}
