package com.example.openairustrealtime.core.model

data class SpeechRequest(
    val model: String,
    val input: String,
    val voice: String,
    val instructions: String,
    val responseFormat: String = "pcm"
)

data class TranscriptionRequest(
    val model: String,
    val responseFormat: String = "text"
)
