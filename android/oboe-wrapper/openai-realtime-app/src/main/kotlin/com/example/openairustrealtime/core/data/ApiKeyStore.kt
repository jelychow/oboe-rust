package com.example.openairustrealtime.core.data

import android.content.Context

class ApiKeyStore(context: Context) {
    private val preferences = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

    fun get(): String = preferences.getString(PREF_OPENAI_API_KEY, "").orEmpty()

    fun hasSavedKey(): Boolean = get().isNotBlank()

    fun save(apiKey: String) {
        preferences.edit().putString(PREF_OPENAI_API_KEY, apiKey.trim()).apply()
    }

    fun clear() {
        preferences.edit().remove(PREF_OPENAI_API_KEY).apply()
    }

    companion object {
        private const val PREFS_NAME = "openai_realtime_settings"
        private const val PREF_OPENAI_API_KEY = "openai_api_key"
    }
}
