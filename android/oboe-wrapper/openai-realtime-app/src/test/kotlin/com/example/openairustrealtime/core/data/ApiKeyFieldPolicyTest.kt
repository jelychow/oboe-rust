package com.example.openairustrealtime.core.data

import org.junit.Assert.assertEquals
import org.junit.Test

class ApiKeyFieldPolicyTest {
    @Test
    fun doesNotRestorePersistedKeyIntoInputField() {
        assertEquals("", ApiKeyFieldPolicy.restoredFieldValue("sk-live-123456"))
    }

    @Test
    fun fallsBackToSavedKeyWhenInputBlank() {
        assertEquals(
            "sk-saved-123456",
            ApiKeyFieldPolicy.resolveApiKey("   ", "  sk-saved-123456  ")
        )
    }

    @Test
    fun prefersFreshlyEnteredKeyOverSavedKey() {
        assertEquals(
            "sk-entered-abcdef",
            ApiKeyFieldPolicy.resolveApiKey("  sk-entered-abcdef  ", "sk-saved-123456")
        )
    }
}
