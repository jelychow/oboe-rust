package com.example.openairustrealtime.core.model

import org.junit.Assert.assertEquals
import org.junit.Test

class RealtimeStatsTest {
    @Test
    fun parseKeepsLegacyStatsContract() {
        val stats = RealtimeStats.parse(
            "Mic sent: 2 chunks / 480 frames. " +
                "Mic dropped: 1 chunks / 240 frames. " +
                "Output played: 3 chunks / 720 frames. " +
                "Levels: mic 0.125, output 0.500."
        )

        assertEquals(2L, stats.inputChunks)
        assertEquals(480L, stats.inputFrames)
        assertEquals(1L, stats.droppedInputChunks)
        assertEquals(240L, stats.droppedInputFrames)
        assertEquals(3L, stats.outputChunks)
        assertEquals(720L, stats.outputFrames)
        assertEquals(0.125f, stats.micLevel, 0.001f)
        assertEquals(0.5f, stats.outputLevel, 0.001f)
        assertEquals(0, stats.inputXRunCount)
        assertEquals(0, stats.outputXRunCount)
    }

    @Test
    fun parseCarriesLowLatencyDiagnostics() {
        val stats = RealtimeStats.parse(
            "Mic sent: 4 chunks / 960 frames. " +
                "Mic dropped: 0 chunks / 0 frames. " +
                "Output played: 5 chunks / 1200 frames. " +
                "Levels: mic 0.250, output 0.750. " +
                "Diagnostics: xruns input 1 / output 2. " +
                "Output latency 37.5 ms / 900 frames pending. " +
                "Buffer: input 384/768 burst 192, output 384/768 burst 192. " +
                "Async error: -899."
        )

        assertEquals(1, stats.inputXRunCount)
        assertEquals(2, stats.outputXRunCount)
        assertEquals(37.5f, stats.outputLatencyMillis, 0.001f)
        assertEquals(900L, stats.outputPendingFrames)
        assertEquals(384, stats.inputBufferSizeFrames)
        assertEquals(768, stats.inputBufferCapacityFrames)
        assertEquals(192, stats.inputBurstFrames)
        assertEquals(384, stats.outputBufferSizeFrames)
        assertEquals(768, stats.outputBufferCapacityFrames)
        assertEquals(192, stats.outputBurstFrames)
        assertEquals(-899, stats.lastAsyncError)
        assertEquals(3, stats.totalXRunCount)
    }
}
