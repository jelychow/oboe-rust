package com.example.openairustrealtime.core.network

import com.example.openairustrealtime.core.model.SpeechRequest
import com.example.openairustrealtime.core.model.TranscriptionRequest
import com.example.openairustrealtime.core.util.SecretRedactor
import java.io.File
import java.io.OutputStream
import java.net.HttpURLConnection
import java.net.URL
import java.util.UUID
import org.json.JSONObject

class OpenAiAudioApi {
    fun synthesizeSpeech(apiKey: String, request: SpeechRequest): ByteArray {
        val payload = JSONObject()
            .put("model", request.model)
            .put("input", request.input)
            .put("voice", request.voice)
            .put("response_format", request.responseFormat)

        if (request.instructions.isNotBlank()) {
            payload.put("instructions", request.instructions)
        }

        val connection = openConnection("/audio/speech", apiKey).apply {
            setRequestProperty("Content-Type", "application/json; charset=utf-8")
            outputStream.use { it.write(payload.toString().toByteArray(Charsets.UTF_8)) }
        }
        return readSuccessfulBytes(connection)
    }

    fun transcribe(apiKey: String, request: TranscriptionRequest, audioFile: File): String {
        val boundary = "OpenAiRustBoundary-${UUID.randomUUID()}"
        val connection = openConnection("/audio/transcriptions", apiKey).apply {
            setRequestProperty("Content-Type", "multipart/form-data; boundary=$boundary")
        }

        connection.outputStream.use { output ->
            writeFormField(output, boundary, "model", request.model)
            writeFormField(output, boundary, "response_format", request.responseFormat)
            writeFileField(output, boundary, "file", audioFile.name, "audio/wav", audioFile)
            output.write("--$boundary--\r\n".toByteArray(Charsets.UTF_8))
        }

        return readSuccessfulBytes(connection).toString(Charsets.UTF_8).trim()
    }

    private fun openConnection(path: String, apiKey: String): HttpURLConnection {
        return (URL("$BASE_URL$path").openConnection() as HttpURLConnection).apply {
            requestMethod = "POST"
            doOutput = true
            connectTimeout = CONNECT_TIMEOUT_MS
            readTimeout = READ_TIMEOUT_MS
            setRequestProperty("Authorization", "Bearer $apiKey")
        }
    }

    private fun readSuccessfulBytes(connection: HttpURLConnection): ByteArray {
        try {
            val code = connection.responseCode
            return if (code in 200..299) {
                connection.inputStream.use { it.readBytes() }
            } else {
                val errorBody = connection.errorStream?.use { it.readBytes() } ?: ByteArray(0)
                throw OpenAiApiException(formatApiError(code, errorBody))
            }
        } finally {
            connection.disconnect()
        }
    }

    private fun formatApiError(code: Int, body: ByteArray): String {
        val raw = body.toString(Charsets.UTF_8).trim()
        val message = runCatching {
            val error = JSONObject(raw).optJSONObject("error")
            error?.optString("message")?.takeIf { it.isNotBlank() }
        }.getOrNull() ?: raw.ifBlank { "request failed" }
        return SecretRedactor.redact("OpenAI API $code: $message")
    }

    private fun writeFormField(output: OutputStream, boundary: String, name: String, value: String) {
        output.write("--$boundary\r\n".toByteArray(Charsets.UTF_8))
        output.write("Content-Disposition: form-data; name=\"$name\"\r\n\r\n".toByteArray(Charsets.UTF_8))
        output.write(value.toByteArray(Charsets.UTF_8))
        output.write("\r\n".toByteArray(Charsets.UTF_8))
    }

    private fun writeFileField(
        output: OutputStream,
        boundary: String,
        name: String,
        filename: String,
        contentType: String,
        file: File
    ) {
        output.write("--$boundary\r\n".toByteArray(Charsets.UTF_8))
        output.write(
            "Content-Disposition: form-data; name=\"$name\"; filename=\"$filename\"\r\n".toByteArray(Charsets.UTF_8)
        )
        output.write("Content-Type: $contentType\r\n\r\n".toByteArray(Charsets.UTF_8))
        file.inputStream().use { it.copyTo(output) }
        output.write("\r\n".toByteArray(Charsets.UTF_8))
    }

    companion object {
        private const val BASE_URL = "https://api.openai.com/v1"
        private const val CONNECT_TIMEOUT_MS = 15_000
        private const val READ_TIMEOUT_MS = 90_000
    }
}

class OpenAiApiException(message: String) : Exception(message)
