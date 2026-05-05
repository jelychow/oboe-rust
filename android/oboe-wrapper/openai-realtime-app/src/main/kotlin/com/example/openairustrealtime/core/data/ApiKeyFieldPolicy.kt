package com.example.openairustrealtime.core.data

object ApiKeyFieldPolicy {
    fun restoredFieldValue(savedKey: String): String = ""

    fun resolveApiKey(input: String, savedKey: String): String {
        return input.trim().ifBlank { savedKey.trim() }
    }

    fun inputHint(hasSavedKey: Boolean): String {
        return if (hasSavedKey) {
            "OpenAI API key (leave blank to reuse saved key)"
        } else {
            "OpenAI API key"
        }
    }
}
