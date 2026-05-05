package com.example.openairustrealtime.core.network

import java.util.ArrayDeque

internal class RealtimeOutputQueue(
    private val maxChunks: Int = 256,
    private val mergeBelowBytes: Int = 2048
) {
    private val chunks = ArrayDeque<ByteArray>()

    @Synchronized
    fun offer(bytes: ByteArray) {
        if (bytes.isEmpty()) return
        if (chunks.isNotEmpty()) {
            val last = chunks.removeLast()
            if (last.size + bytes.size <= mergeBelowBytes) {
                chunks.addLast(concat(last, bytes))
            } else {
                chunks.addLast(last)
                chunks.addLast(bytes)
            }
        } else {
            chunks.addLast(bytes)
        }
        while (chunks.size > maxChunks) {
            val first = chunks.removeFirst()
            val next = if (chunks.isEmpty()) null else chunks.removeFirst()
            if (next == null) {
                chunks.addFirst(first)
                break
            }
            chunks.addFirst(concat(first, next))
        }
    }

    @Synchronized
    fun poll(): ByteArray? = if (chunks.isEmpty()) null else chunks.removeFirst()

    @Synchronized
    fun clear() {
        chunks.clear()
    }

    @Synchronized
    fun size(): Int = chunks.size

    private fun concat(first: ByteArray, second: ByteArray): ByteArray {
        val merged = ByteArray(first.size + second.size)
        System.arraycopy(first, 0, merged, 0, first.size)
        System.arraycopy(second, 0, merged, first.size, second.size)
        return merged
    }
}
