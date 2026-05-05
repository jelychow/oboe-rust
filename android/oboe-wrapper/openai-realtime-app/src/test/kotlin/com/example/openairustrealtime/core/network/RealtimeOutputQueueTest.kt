package com.example.openairustrealtime.core.network

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Test

class RealtimeOutputQueueTest {
    @Test
    fun keepsAllSmallAudioDeltasInOrder() {
        val queue = RealtimeOutputQueue(maxChunks = 256)
        val chunks = (0 until 40).map { byteArrayOf(it.toByte()) }

        chunks.forEach(queue::offer)

        val drained = mutableListOf<Byte>()
        while (true) {
            val next = queue.poll() ?: break
            drained += next.toList()
        }
        assertEquals((0 until 40).toList(), drained.map { it.toInt() and 0xff })
    }

    @Test
    fun mergesAdjacentTinyChunksToReduceQueuePressure() {
        val queue = RealtimeOutputQueue(maxChunks = 4, mergeBelowBytes = 8)

        queue.offer(byteArrayOf(1, 2))
        queue.offer(byteArrayOf(3, 4))
        queue.offer(byteArrayOf(5, 6))

        assertEquals(1, queue.size())
        assertArrayEquals(byteArrayOf(1, 2, 3, 4, 5, 6), queue.poll())
    }
}
