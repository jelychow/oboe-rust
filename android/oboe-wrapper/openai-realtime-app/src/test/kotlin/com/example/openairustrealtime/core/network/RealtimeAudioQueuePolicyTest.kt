package com.example.openairustrealtime.core.network

import java.util.ArrayDeque
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class RealtimeAudioQueuePolicyTest {
    @Test
    fun enqueueOutputAudioKeepsNewestChunksWithinLowLatencyBudget() {
        val queue = ArrayDeque<ByteArray>()

        repeat(RealtimeAudioQueuePolicy.OUTPUT_AUDIO_QUEUE_CAPACITY + 3) { index ->
            RealtimeAudioQueuePolicy.enqueueOutputAudio(queue, byteArrayOf(index.toByte()))
        }

        assertEquals(RealtimeAudioQueuePolicy.OUTPUT_AUDIO_QUEUE_CAPACITY, queue.size)
        assertEquals(3, queue.peekFirst()[0].toInt())
        assertEquals(
            RealtimeAudioQueuePolicy.OUTPUT_AUDIO_QUEUE_CAPACITY + 2,
            queue.peekLast()[0].toInt()
        )
    }

    @Test
    fun maxQueuedOutputAudioStaysInsideRealtimeBudget() {
        assertTrue(RealtimeAudioQueuePolicy.maxQueuedOutputMillis() <= 100)
    }
}
