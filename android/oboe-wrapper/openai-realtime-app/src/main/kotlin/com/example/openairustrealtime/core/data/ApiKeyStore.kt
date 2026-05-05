package com.example.openairustrealtime.core.data

import android.content.Context
import android.content.SharedPreferences
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Log
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

class ApiKeyStore(context: Context) {
    private val prefs: SharedPreferences =
        context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
    @Volatile private var sessionApiKey = ""

    fun get(): String {
        if (sessionApiKey.isNotBlank()) return sessionApiKey
        val encoded = prefs.getString(KEY_API_KEY, null) ?: return ""
        val record = EncryptedSecretRecord.decode(encoded) ?: run {
            clear()
            return ""
        }
        return runCatching {
            decrypt(record).also { sessionApiKey = it }
        }.getOrElse { error ->
            Log.w(TAG, "Unable to decrypt stored API key; clearing persisted key.", error)
            clear()
            ""
        }
    }

    fun hasSavedKey(): Boolean = get().isNotBlank()

    fun save(apiKey: String) {
        val normalized = apiKey.trim()
        if (normalized.isBlank()) {
            clear()
            return
        }
        sessionApiKey = normalized
        runCatching {
            check(prefs.edit().putString(KEY_API_KEY, encrypt(normalized).encode()).commit()) {
                "SharedPreferences commit failed."
            }
        }.onFailure { error ->
            Log.w(TAG, "Unable to persist encrypted API key; keeping it for this process only.", error)
        }
    }

    fun clear() {
        sessionApiKey = ""
        prefs.edit().remove(KEY_API_KEY).apply()
    }

    private fun encrypt(value: String): EncryptedSecretRecord {
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.ENCRYPT_MODE, secretKey())
        return EncryptedSecretRecord(
            iv = cipher.iv,
            cipherText = cipher.doFinal(value.toByteArray(Charsets.UTF_8))
        )
    }

    private fun decrypt(record: EncryptedSecretRecord): String {
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.DECRYPT_MODE, secretKey(), GCMParameterSpec(GCM_TAG_BITS, record.iv))
        return cipher.doFinal(record.cipherText).toString(Charsets.UTF_8)
    }

    private fun secretKey(): SecretKey {
        val keyStore = KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }
        val existing = keyStore.getKey(KEY_ALIAS, null) as? SecretKey
        if (existing != null) return existing

        val keyGenerator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, ANDROID_KEYSTORE)
        keyGenerator.init(
            KeyGenParameterSpec.Builder(
                KEY_ALIAS,
                KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
            )
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setKeySize(KEY_SIZE_BITS)
                .setRandomizedEncryptionRequired(true)
                .build()
        )
        return keyGenerator.generateKey()
    }

    private companion object {
        private const val TAG = "ApiKeyStore"
        private const val PREFS_NAME = "openai_voice_secure_prefs"
        private const val KEY_API_KEY = "encrypted_api_key"
        private const val KEY_ALIAS = "openai_voice_api_key"
        private const val ANDROID_KEYSTORE = "AndroidKeyStore"
        private const val TRANSFORMATION = "AES/GCM/NoPadding"
        private const val GCM_TAG_BITS = 128
        private const val KEY_SIZE_BITS = 256
    }
}
