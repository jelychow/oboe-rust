package com.example.openairustrealtime.core.data

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class EncryptedSecretRecordTest {
    @Test
    fun encodesAndDecodesPersistedSecret() {
        val record = EncryptedSecretRecord(
            iv = byteArrayOf(1, 2, 3, 4),
            cipherText = byteArrayOf(5, 6, 7, 8)
        )

        val parsed = EncryptedSecretRecord.decode(record.encode())

        assertEquals("v1:AQIDBA==:BQYHCA==", record.encode())
        assertArrayEquals(record.iv, parsed?.iv)
        assertArrayEquals(record.cipherText, parsed?.cipherText)
    }

    @Test
    fun rejectsUnknownOrMalformedPersistedSecret() {
        assertNull(EncryptedSecretRecord.decode(""))
        assertNull(EncryptedSecretRecord.decode("v2:AQ==:Ag=="))
        assertNull(EncryptedSecretRecord.decode("v1:not-base64:Ag=="))
        assertNull(EncryptedSecretRecord.decode("v1:AQ=="))
    }
}
