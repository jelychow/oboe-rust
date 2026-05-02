package com.example.openairustrealtime.core.util

object SecretRedactor {
    private val keyPattern = Regex("sk-[A-Za-z0-9_\\-]{6,}")

    fun redact(value: String): String = keyPattern.replace(value) { "sk-***" }
}
