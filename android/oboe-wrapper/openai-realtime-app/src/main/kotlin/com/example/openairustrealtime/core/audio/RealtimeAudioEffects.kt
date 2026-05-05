package com.example.openairustrealtime.core.audio

import android.content.Context
import android.media.AudioManager
import android.media.audiofx.AcousticEchoCanceler
import android.media.audiofx.NoiseSuppressor
import android.util.Log
import com.example.openairustrealtime.core.util.AppLog
import com.google.oboe.AudioStream
import com.google.oboe.AudioStreamBuilder

class RealtimeAudioEffects(context: Context) {
    private val communicationAudio = runCatching {
        CommunicationAudioController(
            backend = AndroidCommunicationAudioBackend(context.applicationContext),
            communicationMode = AudioManager.MODE_IN_COMMUNICATION
        )
    }.getOrNull()

    fun enterCommunicationMode() {
        val controller = communicationAudio
        if (controller == null) {
            Log.w(TAG, "Realtime communication audio mode unavailable")
            return
        }
        runCatching {
            controller.activate()
        }.onSuccess { device ->
            AppLog.i(TAG, "Realtime communication audio route=${device?.name ?: "none"}")
        }.onFailure { error ->
            Log.w(TAG, "Failed to enable realtime communication audio mode", error)
        }
    }

    fun leaveCommunicationMode() {
        val controller = communicationAudio ?: return
        runCatching {
            controller.deactivate()
        }.onSuccess {
            AppLog.i(TAG, "Realtime communication audio mode restored")
        }.onFailure { error ->
            Log.w(TAG, "Failed to restore realtime communication audio mode", error)
        }
    }

    fun attachCaptureEffects(stream: AudioStream): CaptureEffects {
        val sessionId = RealtimeOboeCompat.getSessionId(stream)
        if (!RealtimeAudioEffectPolicy.canAttachToSession(sessionId)) {
            Log.w(TAG, "Realtime capture effects unavailable sessionId=$sessionId")
            return CaptureEffects(sessionId = sessionId)
        }

        val echoCanceler = createEchoCanceler(sessionId)
        val noiseSuppressor = createNoiseSuppressor(sessionId)
        val effects = CaptureEffects(
            sessionId = sessionId,
            echoCanceler = echoCanceler,
            echoCancelerEnabled = effectEnabled(echoCanceler),
            noiseSuppressor = noiseSuppressor,
            noiseSuppressorEnabled = effectEnabled(noiseSuppressor)
        )
        AppLog.i(TAG, "Realtime capture effects ${effects.summary()}")
        return effects
    }

    private fun createEchoCanceler(sessionId: Int): AcousticEchoCanceler? {
        if (!runCatching { AcousticEchoCanceler.isAvailable() }.getOrDefault(false)) {
            Log.w(TAG, "AcousticEchoCanceler is not available on this device")
            return null
        }
        return runCatching {
            AcousticEchoCanceler.create(sessionId)?.also { it.setEnabled(true) }
        }.onFailure { error ->
            Log.w(TAG, "Failed to create AcousticEchoCanceler sessionId=$sessionId", error)
        }.getOrNull()
    }

    private fun createNoiseSuppressor(sessionId: Int): NoiseSuppressor? {
        if (!runCatching { NoiseSuppressor.isAvailable() }.getOrDefault(false)) {
            AppLog.i(TAG, "NoiseSuppressor is not available on this device")
            return null
        }
        return runCatching {
            NoiseSuppressor.create(sessionId)?.also { it.setEnabled(true) }
        }.onFailure { error ->
            Log.w(TAG, "Failed to create NoiseSuppressor sessionId=$sessionId", error)
        }.getOrNull()
    }

    private fun effectEnabled(effect: android.media.audiofx.AudioEffect?): Boolean {
        return effect != null && runCatching { effect.enabled }.getOrDefault(false)
    }

    class CaptureEffects(
        private val sessionId: Int,
        private val echoCanceler: AcousticEchoCanceler? = null,
        private val echoCancelerEnabled: Boolean = false,
        private val noiseSuppressor: NoiseSuppressor? = null,
        private val noiseSuppressorEnabled: Boolean = false
    ) : AutoCloseable {
        fun summary(): String {
            return "sessionId=$sessionId aec=$echoCancelerEnabled ns=$noiseSuppressorEnabled"
        }

        override fun close() {
            runCatching { echoCanceler?.release() }
            runCatching { noiseSuppressor?.release() }
        }
    }

    private companion object {
        private const val TAG = "RealtimeAudioEffects"
    }
}

object RealtimeAudioEffectPolicy {
    fun canAttachToSession(sessionId: Int): Boolean = sessionId > 0
}

object RealtimeOboeCompat {
    private const val TAG = "RealtimeOboeCompat"
    private const val SESSION_ID_NONE = -1

    fun configureVoiceCommunicationInput(
        builder: AudioStreamBuilder,
        allocateSession: Boolean = true
    ) {
        val inputPresetConfigured = setVoiceCommunicationInputPreset(builder)
        val sessionConfigured = allocateSession && setAllocatedSessionId(builder)
        AppLog.i(
            TAG,
            "Realtime Oboe input config inputPreset=$inputPresetConfigured sessionAllocate=$sessionConfigured"
        )
    }

    fun configureVoiceCommunicationOutput(builder: AudioStreamBuilder) {
        val usageConfigured = setVoiceCommunicationUsage(builder)
        val contentTypeConfigured = setSpeechContentType(builder)
        AppLog.i(
            TAG,
            "Realtime Oboe output config usage=$usageConfigured contentType=$contentTypeConfigured"
        )
    }

    fun getSessionId(stream: AudioStream): Int {
        return runCatching {
            val method = stream.javaClass.getMethod("getSessionId")
            (method.invoke(stream) as? Number)?.toInt() ?: SESSION_ID_NONE
        }.onFailure { error ->
            Log.w(TAG, "Oboe wrapper session id API unavailable: ${error.compatMessage()}")
        }.getOrDefault(SESSION_ID_NONE)
    }

    private fun setVoiceCommunicationInputPreset(builder: AudioStreamBuilder): Boolean {
        return runCatching {
            val presetClass = Class.forName("com.google.oboe.InputPreset")
            val voiceCommunication = presetClass.enumConstants
                .first { (it as Enum<*>).name == "VOICE_COMMUNICATION" }
            val method = builder.javaClass.getMethod("setInputPreset", presetClass)
            method.invoke(builder, voiceCommunication)
        }.onFailure { error ->
            Log.w(TAG, "Oboe wrapper input preset API unavailable: ${error.compatMessage()}")
        }.isSuccess
    }

    private fun setAllocatedSessionId(builder: AudioStreamBuilder): Boolean {
        return runCatching {
            val sessionId = runCatching {
                builder.javaClass.getField("SESSION_ID_ALLOCATE").getInt(null)
            }.getOrDefault(0)
            val method = builder.javaClass.getMethod("setSessionId", Integer.TYPE)
            method.invoke(builder, sessionId)
        }.onFailure { error ->
            Log.w(TAG, "Oboe wrapper session allocation API unavailable: ${error.compatMessage()}")
        }.isSuccess
    }

    private fun setVoiceCommunicationUsage(builder: AudioStreamBuilder): Boolean {
        return runCatching {
            val usageClass = Class.forName("com.google.oboe.Usage")
            val voiceCommunication = usageClass.enumConstants
                .first { (it as Enum<*>).name == "VOICE_COMMUNICATION" }
            val method = builder.javaClass.getMethod("setUsage", usageClass)
            method.invoke(builder, voiceCommunication)
        }.onFailure { error ->
            Log.w(TAG, "Oboe wrapper output usage API unavailable: ${error.compatMessage()}")
        }.isSuccess
    }

    private fun setSpeechContentType(builder: AudioStreamBuilder): Boolean {
        return runCatching {
            val contentTypeClass = Class.forName("com.google.oboe.ContentType")
            val speech = contentTypeClass.enumConstants
                .first { (it as Enum<*>).name == "SPEECH" }
            val method = builder.javaClass.getMethod("setContentType", contentTypeClass)
            method.invoke(builder, speech)
        }.onFailure { error ->
            Log.w(TAG, "Oboe wrapper output content type API unavailable: ${error.compatMessage()}")
        }.isSuccess
    }

    private fun Throwable.compatMessage(): String {
        return "${javaClass.simpleName}: ${message.orEmpty()}"
    }
}
