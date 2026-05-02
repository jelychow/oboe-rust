package com.example.openairustrealtime.core.network

import com.example.openairustrealtime.core.audio.PcmAudio
import java.util.Base64
import org.json.JSONArray
import org.json.JSONObject

internal sealed class RealtimeServerEvent {
    data class AudioDelta(val bytes: ByteArray) : RealtimeServerEvent() {
        override fun equals(other: Any?): Boolean {
            return other is AudioDelta && bytes.contentEquals(other.bytes)
        }

        override fun hashCode(): Int = bytes.contentHashCode()
    }

    data class TranscriptDelta(val text: String) : RealtimeServerEvent()
    data class StatusChanged(val status: String) : RealtimeServerEvent()
    data class Error(val message: String) : RealtimeServerEvent()
    object TranscriptDone : RealtimeServerEvent()
    object Ignored : RealtimeServerEvent()
}

internal object RealtimeProtocol {
    private const val SAMPLE_RATE = PcmAudio.SAMPLE_RATE
    private const val DEFAULT_VOICE = "marin"

    fun sessionUpdate(model: String, instructions: String): String {
        return JSONObject()
            .put("type", "session.update")
            .put(
                "session",
                JSONObject()
                    .put("type", "realtime")
                    .put("model", model)
                    .put("instructions", instructions)
                    .put("output_modalities", JSONArray().put("audio"))
                    .put(
                        "audio",
                        JSONObject()
                            .put(
                                "input",
                                JSONObject()
                                    .put(
                                        "format",
                                        JSONObject()
                                            .put("type", "audio/pcm")
                                            .put("rate", SAMPLE_RATE)
                                    )
                                    .put(
                                        "turn_detection",
                                        JSONObject().put("type", "semantic_vad")
                                    )
                            )
                            .put(
                                "output",
                                JSONObject()
                                    .put(
                                        "format",
                                        JSONObject()
                                            .put("type", "audio/pcm")
                                            .put("rate", SAMPLE_RATE)
                                    )
                                    .put("voice", DEFAULT_VOICE)
                            )
                    )
            )
            .toString()
    }

    fun audioAppend(audio: FloatArray, sampleCount: Int): String {
        val pcm = PcmAudio.floatToPcm16Bytes(audio, sampleCount)
        return JSONObject()
            .put("type", "input_audio_buffer.append")
            .put("audio", Base64.getEncoder().encodeToString(pcm))
            .toString()
    }

    fun parseServerEvent(text: String): RealtimeServerEvent {
        val value = JSONObject(text)
        return when (val type = value.optString("type")) {
            "session.created", "session.updated" -> RealtimeServerEvent.StatusChanged("Connected")
            "input_audio_buffer.speech_started" -> RealtimeServerEvent.StatusChanged("Listening")
            "input_audio_buffer.speech_stopped",
            "input_audio_buffer.committed" -> RealtimeServerEvent.StatusChanged("Thinking")
            "response.created",
            "response.output_item.added",
            "response.output_item.created" -> RealtimeServerEvent.StatusChanged("Responding")
            "response.output_audio.delta",
            "response.audio.delta" -> audioDelta(value)
            "response.output_audio_transcript.delta",
            "response.audio_transcript.delta",
            "response.output_text.delta",
            "response.text.delta" -> RealtimeServerEvent.TranscriptDelta(value.optString("delta"))
            "response.output_audio_transcript.done",
            "response.output_text.done",
            "response.done" -> RealtimeServerEvent.TranscriptDone
            "error" -> RealtimeServerEvent.Error(errorMessage(value))
            else -> {
                if (type.isBlank()) {
                    RealtimeServerEvent.Error("Realtime API returned an event without a type.")
                } else {
                    RealtimeServerEvent.Ignored
                }
            }
        }
    }

    private fun audioDelta(value: JSONObject): RealtimeServerEvent {
        val delta = value.optString("delta")
        if (delta.isBlank()) return RealtimeServerEvent.Ignored
        return RealtimeServerEvent.AudioDelta(Base64.getDecoder().decode(delta))
    }

    private fun errorMessage(value: JSONObject): String {
        return value.optJSONObject("error")
            ?.optString("message")
            ?.takeIf { it.isNotBlank() }
            ?: value.optString("message").ifBlank { "Realtime API returned an error." }
    }
}
