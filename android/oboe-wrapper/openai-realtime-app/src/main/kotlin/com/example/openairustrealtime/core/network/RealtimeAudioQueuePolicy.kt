package com.example.openairustrealtime.core.network

import com.example.openairustrealtime.core.audio.PcmAudio
import java.util.ArrayDeque

internal object RealtimeAudioQueuePolicy {
    const val OUTPUT_AUDIO_QUEUE_CAPACITY = 4

    fun enqueueOutputAudio(queue: ArrayDeque<ByteArray>, audio: ByteArray) {
        while (queue.size >= OUTPUT_AUDIO_QUEUE_CAPACITY) {
            queue.pollFirst()
        }
        queue.addLast(audio)
    }

    fun maxQueuedOutputMillis(): Int = OUTPUT_AUDIO_QUEUE_CAPACITY * PcmAudio.CHUNK_DURATION_MILLIS
}
