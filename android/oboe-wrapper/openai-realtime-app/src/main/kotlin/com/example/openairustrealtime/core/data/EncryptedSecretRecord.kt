package com.example.openairustrealtime.core.data

import java.util.Base64

internal data class EncryptedSecretRecord(
    val iv: ByteArray,
    val cipherText: ByteArray
) {
    override fun equals(other: Any?): Boolean {
        return other is EncryptedSecretRecord &&
            iv.contentEquals(other.iv) &&
            cipherText.contentEquals(other.cipherText)
    }

    override fun hashCode(): Int {
        var result = iv.contentHashCode()
        result = 31 * result + cipherText.contentHashCode()
        return result
    }

    fun encode(): String {
        return listOf(
            VERSION,
            Base64.getEncoder().encodeToString(iv),
            Base64.getEncoder().encodeToString(cipherText)
        ).joinToString(":")
    }

    companion object {
        private const val VERSION = "v1"

        fun decode(value: String): EncryptedSecretRecord? {
            val parts = value.split(":")
            if (parts.size != 3 || parts[0] != VERSION) return null
            return runCatching {
                EncryptedSecretRecord(
                    iv = Base64.getDecoder().decode(parts[1]),
                    cipherText = Base64.getDecoder().decode(parts[2])
                )
            }.getOrNull()
        }
    }
}
