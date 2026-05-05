package com.example.openairustrealtime.core.audio

import org.junit.Assert.assertEquals
import org.junit.Test

class PcmAudioTimingTest {
    @Test
    fun realtimeTimeoutMatchesOneAudioChunk() {
        val chunkMillis = PcmAudio.FRAMES_PER_CHUNK * 1_000 / PcmAudio.SAMPLE_RATE

        assertEquals(chunkMillis, PcmAudio.CHUNK_DURATION_MILLIS)
        assertEquals(
            PcmAudio.CHUNK_DURATION_MILLIS * 1_000_000L,
            PcmAudio.REALTIME_IO_TIMEOUT_NANOS
        )
    }
}
