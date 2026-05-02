package com.example.openairustrealtime.core.network

import java.util.Base64
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class RealtimeProtocolTest {
    @Test
    fun sessionUpdateUsesRealtimeAudioShape() {
        val event = JSONObject(
            RealtimeProtocol.sessionUpdate(
                model = "gpt-realtime",
                instructions = "short replies"
            )
        )

        assertEquals("session.update", event.getString("type"))
        val session = event.getJSONObject("session")
        assertEquals("realtime", session.getString("type"))
        assertEquals("gpt-realtime", session.getString("model"))
        assertEquals("short replies", session.getString("instructions"))
        assertEquals(24_000, session.getJSONObject("audio").getJSONObject("input").getJSONObject("format").getInt("rate"))
        assertEquals(24_000, session.getJSONObject("audio").getJSONObject("output").getJSONObject("format").getInt("rate"))
        assertEquals("marin", session.getJSONObject("audio").getJSONObject("output").getString("voice"))
    }

    @Test
    fun audioAppendEncodesPcm16Payload() {
        val event = JSONObject(RealtimeProtocol.audioAppend(floatArrayOf(-1f, 0f, 1f), 3))

        assertEquals("input_audio_buffer.append", event.getString("type"))
        val bytes = Base64.getDecoder().decode(event.getString("audio"))
        assertEquals(listOf(0x00, 0x80, 0x00, 0x00, 0xff, 0x7f), bytes.map { it.toInt() and 0xff })
    }

    @Test
    fun serverEventsParseAudioTranscriptStatusAndError() {
        val audio = RealtimeProtocol.parseServerEvent("""{"type":"response.output_audio.delta","delta":"AQID"}""")
        val transcript = RealtimeProtocol.parseServerEvent("""{"type":"response.output_text.delta","delta":"hi"}""")
        val status = RealtimeProtocol.parseServerEvent("""{"type":"input_audio_buffer.speech_started"}""")
        val error = RealtimeProtocol.parseServerEvent("""{"type":"error","error":{"message":"bad key"}}""")

        assertTrue(audio is RealtimeServerEvent.AudioDelta)
        assertEquals(listOf(1, 2, 3), (audio as RealtimeServerEvent.AudioDelta).bytes.map { it.toInt() and 0xff })
        assertEquals(RealtimeServerEvent.TranscriptDelta("hi"), transcript)
        assertEquals(RealtimeServerEvent.StatusChanged("Listening"), status)
        assertEquals(RealtimeServerEvent.Error("bad key"), error)
    }
}
