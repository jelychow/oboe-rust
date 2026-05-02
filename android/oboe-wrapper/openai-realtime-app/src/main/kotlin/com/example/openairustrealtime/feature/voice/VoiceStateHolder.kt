package com.example.openairustrealtime.feature.voice

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.os.Handler
import android.os.Looper
import com.example.openairustrealtime.core.data.OpenAiVoiceRepository
import com.example.openairustrealtime.core.model.RealtimeStats
import com.example.openairustrealtime.core.model.SpeechRequest
import com.example.openairustrealtime.core.model.TranscriptionRequest
import com.example.openairustrealtime.core.model.VoiceMode
import com.example.openairustrealtime.core.model.VoiceUiState
import com.example.openairustrealtime.core.util.SecretRedactor
import java.io.Closeable
import java.util.Locale
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors

class VoiceStateHolder(context: Context) : Closeable {
    private val appContext = context.applicationContext
    private val repository = OpenAiVoiceRepository(appContext)
    private val mainHandler = Handler(Looper.getMainLooper())
    private val executor: ExecutorService = Executors.newSingleThreadExecutor()
    private var observer: ((VoiceUiState) -> Unit)? = null
    private var state = VoiceUiState(
        savedKeyPresent = repository.apiKeyStore.hasSavedKey(),
        micPermissionGranted = hasRecordAudioPermission()
    )
    private var lastRealtimeStatus = ""
    private var lastRealtimeError = ""
    private var lastStats = RealtimeStats()
    private var lastTranscriptLength = 0

    private val pollRunnable = object : Runnable {
        override fun run() {
            refreshRealtimeState()
            val audioActive = state.realtimeRunning || state.recording || state.status == "Playing"
            mainHandler.postDelayed(this, if (audioActive) 120L else 1_000L)
        }
    }

    fun observe(observer: (VoiceUiState) -> Unit) {
        this.observer = observer
        observer(state)
    }

    fun savedApiKey(): String = repository.apiKeyStore.get()

    fun startPolling() {
        mainHandler.removeCallbacks(pollRunnable)
        mainHandler.post(pollRunnable)
    }

    fun setMicPermission(granted: Boolean) {
        update { it.copy(micPermissionGranted = granted) }
    }

    fun selectMode(mode: VoiceMode) {
        update {
            it.copy(
                selectedMode = mode,
                statusDetail = modeDetail(mode),
                resultTitle = modeResultTitle(mode)
            )
        }
        addEvent("Mode: ${mode.label}")
    }

    fun saveKey(apiKey: String) {
        val normalized = apiKey.trim()
        if (normalized.isBlank()) {
            fail("Enter an OpenAI API key before saving.")
            return
        }
        repository.apiKeyStore.save(normalized)
        update { it.copy(savedKeyPresent = true, lastError = "") }
        addEvent("API key saved")
    }

    fun clearKey() {
        repository.apiKeyStore.clear()
        update { it.copy(savedKeyPresent = false, lastError = "") }
        addEvent("Saved API key cleared")
    }

    fun runTts(apiKey: String, text: String, model: String, voice: String, instructions: String) {
        val normalizedKey = apiKey.trim()
        val normalizedText = text.trim()
        if (!validateApiKey(normalizedKey)) return
        if (normalizedText.isBlank()) {
            fail("Enter text for TTS.")
            return
        }
        if (state.realtimeRunning) {
            fail("Stop realtime translate before running TTS.")
            return
        }

        repository.apiKeyStore.save(normalizedKey)
        val request = SpeechRequest(
            model = model.ifBlank { DEFAULT_TTS_MODEL },
            input = normalizedText,
            voice = voice.ifBlank { DEFAULT_VOICE },
            instructions = instructions.trim()
        )

        update {
            it.copy(
                selectedMode = VoiceMode.TTS,
                savedKeyPresent = true,
                busy = true,
                status = "Synthesizing",
                statusDetail = "Calling OpenAI speech and preparing playback.",
                resultTitle = "TTS output",
                resultText = "Generating speech...",
                lastError = ""
            )
        }
        addEvent("TTS request started")

        executor.execute {
            runCatching { repository.synthesizeAndPlay(normalizedKey, request) }
                .onSuccess { file ->
                    postUpdate {
                        it.copy(
                            busy = false,
                            status = "Playing",
                            statusDetail = "Generated audio is playing from device output.",
                            resultText = "Generated PCM audio: ${file.name} (${file.length()} bytes), playing through native oboe.",
                            lastError = ""
                        )
                    }
                    postEvent("TTS playback started")
                }
                .onFailure { throwable ->
                    postFailure(throwable)
                }
        }
    }

    fun startAsrRecording() {
        if (!hasRecordAudioPermission()) {
            fail("Microphone permission is required for ASR.")
            return
        }
        if (state.realtimeRunning) {
            fail("Stop realtime translate before recording ASR.")
            return
        }
        if (state.recording || state.busy) return

        update {
            it.copy(
                selectedMode = VoiceMode.ASR,
                recording = true,
                status = "Recording",
                statusDetail = "Capturing microphone audio into a WAV file.",
                resultTitle = "ASR transcript",
                resultText = "Recording...",
                lastError = ""
            )
        }
        addEvent("ASR recording started")

        executor.execute {
            runCatching { repository.startRecording() }
                .onFailure { throwable ->
                    postUpdate {
                        it.copy(recording = false, status = "Error", statusDetail = "Recording failed.")
                    }
                    postFailure(throwable)
                }
        }
    }

    fun stopAsrAndTranscribe(apiKey: String, model: String) {
        val normalizedKey = apiKey.trim()
        if (!state.recording) {
            fail("Start recording before transcribing.")
            return
        }
        if (normalizedKey.isBlank()) {
            update {
                it.copy(
                    status = "Recording",
                    statusDetail = "Enter an API key, then stop and transcribe.",
                    lastError = "Enter an OpenAI API key."
                )
            }
            addEvent("Error: Enter an OpenAI API key")
            return
        }

        repository.apiKeyStore.save(normalizedKey)
        update {
            it.copy(
                savedKeyPresent = true,
                recording = false,
                busy = true,
                status = "Transcribing",
                statusDetail = "Uploading recorded WAV audio to OpenAI transcription.",
                resultText = "Transcribing...",
                lastError = ""
            )
        }
        addEvent("ASR transcription started")

        executor.execute {
            runCatching {
                repository.stopRecordingAndTranscribe(
                    normalizedKey,
                    TranscriptionRequest(model = model.ifBlank { DEFAULT_ASR_MODEL })
                )
            }.onSuccess { transcript ->
                postUpdate {
                    it.copy(
                        busy = false,
                        status = "Transcribed",
                        statusDetail = "OpenAI returned a transcript.",
                        resultText = transcript.ifBlank { "(empty transcript)" },
                        lastError = ""
                    )
                }
                postEvent("ASR transcript updated")
            }.onFailure { throwable ->
                postFailure(throwable) { it.copy(recording = false, busy = false) }
            }
        }
    }

    fun cancelAsrRecording() {
        if (!state.recording) return
        executor.execute { repository.cancelRecording() }
        update {
            it.copy(
                recording = false,
                busy = false,
                status = "Ready",
                statusDetail = "Recording cancelled.",
                resultText = "Recording cancelled."
            )
        }
        addEvent("ASR recording cancelled")
    }

    fun startRealtimeChat(apiKey: String, model: String, instructions: String) {
        val normalizedKey = apiKey.trim()
        if (!validateApiKey(normalizedKey)) return
        if (!hasRecordAudioPermission()) {
            fail("Microphone permission is required for realtime.")
            return
        }
        if (state.realtimeRunning || state.busy) return

        val realtimeModel = model.trim().ifBlank { DEFAULT_REALTIME_MODEL }
        val realtimeInstructions = instructions.trim().ifBlank { DEFAULT_REALTIME_INSTRUCTIONS }
        startRealtimeSession(
            apiKey = normalizedKey,
            model = realtimeModel,
            instructions = realtimeInstructions,
            mode = VoiceMode.REALTIME_CHAT,
            resultTitle = "Realtime transcript",
            resultText = "Speak to the device after the session connects.",
            startingEvent = "Realtime session starting",
            activeDetail = "Realtime voice assistant is active."
        )
    }

    fun startRealtimeTranslate(
        apiKey: String,
        model: String,
        targetLanguage: String,
        extraInstructions: String
    ) {
        val normalizedKey = apiKey.trim()
        if (!validateApiKey(normalizedKey)) return
        if (!hasRecordAudioPermission()) {
            fail("Microphone permission is required for realtime translate.")
            return
        }
        if (state.realtimeRunning || state.busy) return

        val target = targetLanguage.trim().ifBlank { DEFAULT_TARGET_LANGUAGE }
        val realtimeModel = model.trim().ifBlank { DEFAULT_REALTIME_MODEL }
        val instructions = buildTranslateInstructions(target, extraInstructions)
        startRealtimeSession(
            apiKey = normalizedKey,
            model = realtimeModel,
            instructions = instructions,
            mode = VoiceMode.REALTIME_TRANSLATE,
            resultTitle = "Realtime translation",
            resultText = "Speak to the device after the translation session connects.",
            startingEvent = "Realtime translate starting",
            activeDetail = "Realtime translation is active."
        )
    }

    private fun startRealtimeSession(
        apiKey: String,
        model: String,
        instructions: String,
        mode: VoiceMode,
        resultTitle: String,
        resultText: String,
        startingEvent: String,
        activeDetail: String
    ) {
        repository.apiKeyStore.save(apiKey)
        lastStats = RealtimeStats()
        lastRealtimeStatus = ""
        lastRealtimeError = ""
        lastTranscriptLength = 0

        update {
            it.copy(
                selectedMode = mode,
                savedKeyPresent = true,
                busy = true,
                realtimeRunning = true,
                status = "Connecting",
                statusDetail = "Opening Rust oboe Realtime session.",
                resultTitle = resultTitle,
                resultText = resultText,
                lastError = "",
                stats = RealtimeStats(),
                micLevel = 0f,
                outputLevel = 0f
            )
        }
        addEvent(startingEvent)

        executor.execute {
            val result = runCatching {
                repository.startRealtime(apiKey, model, instructions)
            }
            result.onSuccess { code ->
                postUpdate {
                    if (code == 0) {
                        it.copy(busy = false, statusDetail = activeDetail)
                    } else {
                        it.copy(
                            busy = false,
                            realtimeRunning = false,
                            status = "Error",
                            statusDetail = "Native Realtime start failed.",
                            lastError = "Native Realtime start returned $code"
                        )
                    }
                }
                if (code != 0) postEvent("Realtime start failed: $code")
            }.onFailure { throwable ->
                postFailure(throwable) { it.copy(busy = false, realtimeRunning = false) }
            }
        }
    }

    fun stopRealtime() {
        if (!state.realtimeRunning && !state.busy) return
        update {
            it.copy(
                realtimeRunning = false,
                busy = false,
                status = "Stopping",
                statusDetail = "Closing Rust oboe Realtime session."
            )
        }
        addEvent("Realtime session stopping")
        executor.execute {
            repository.stopRealtime()
            mainHandler.post { refreshRealtimeState() }
        }
    }

    private fun refreshRealtimeState() {
        if (observer == null) return
        val rawStatus = repository.realtimeStatus()
        val status = rawStatus.ifBlank { if (state.realtimeRunning) "Connecting" else "Stopped" }
        val stats = RealtimeStats.parse(repository.realtimeStats())
        val transcript = SecretRedactor.redact(repository.realtimeTranscript())
        val error = SecretRedactor.redact(repository.realtimeError())

        val statusChanged = status != lastRealtimeStatus
        if (statusChanged) {
            addEvent("Status: $status")
            lastRealtimeStatus = status
        }
        if (error.isNotBlank() && error != lastRealtimeError) {
            addEvent("Error: $error")
            lastRealtimeError = error
        }

        val inputDelta = (stats.inputChunks - lastStats.inputChunks).coerceAtLeast(0L)
        val droppedDelta = (stats.droppedInputChunks - lastStats.droppedInputChunks).coerceAtLeast(0L)
        val outputDelta = (stats.outputChunks - lastStats.outputChunks).coerceAtLeast(0L)
        if (inputDelta > 0L) addEvent("Mic +$inputDelta chunks")
        if (droppedDelta > 0L) addEvent("Mic dropped +$droppedDelta chunks")
        if (outputDelta > 0L) addEvent("Audio +$outputDelta chunks")
        val micLevel = if (inputDelta > 0L) stats.micLevel else 0f
        val outputLevel = if (outputDelta > 0L) stats.outputLevel else 0f

        val realtimeTitle = if (state.selectedMode == VoiceMode.REALTIME_TRANSLATE) {
            "Realtime translation"
        } else {
            "Realtime transcript"
        }
        val idleText = if (state.selectedMode == VoiceMode.REALTIME_TRANSLATE) {
            "Listening for translated speech..."
        } else {
            "Listening for realtime conversation..."
        }

        val isRealtimeMode = state.selectedMode == VoiceMode.REALTIME_CHAT ||
            state.selectedMode == VoiceMode.REALTIME_TRANSLATE
        val nativeHasActiveStatus = rawStatus.isNotBlank() && rawStatus != "Stopped"
        val shouldShowRealtime = isRealtimeMode ||
            state.realtimeRunning ||
            nativeHasActiveStatus ||
            error.isNotBlank()

        if (shouldShowRealtime) {
            val stoppedByNative = status == "Stopped" || status == "Error"
            val nextRunning = state.realtimeRunning && !stoppedByNative
            update {
                it.copy(
                    realtimeRunning = nextRunning,
                    busy = if (stoppedByNative) false else it.busy,
                    status = status,
                    statusDetail = statusDetail(status),
                    resultTitle = realtimeTitle,
                    resultText = transcript.ifBlank {
                        if (nextRunning) idleText else it.resultText
                    },
                    lastError = error,
                    stats = stats,
                    micLevel = micLevel,
                    outputLevel = outputLevel
                )
            }
        } else if (state.recording || state.status == "Playing" || stats != lastStats) {
            update {
                it.copy(
                    stats = stats,
                    micLevel = micLevel,
                    outputLevel = outputLevel
                )
            }
        }

        if (transcript.length > lastTranscriptLength) {
            addEvent("Transcript updated")
        }
        lastTranscriptLength = transcript.length
        lastStats = stats
    }

    private fun validateApiKey(apiKey: String): Boolean {
        if (apiKey.isBlank()) {
            fail("Enter an OpenAI API key.")
            return false
        }
        return true
    }

    private fun hasRecordAudioPermission(): Boolean {
        return appContext.checkSelfPermission(Manifest.permission.RECORD_AUDIO) ==
            PackageManager.PERMISSION_GRANTED
    }

    private fun buildTranslateInstructions(targetLanguage: String, extraInstructions: String): String {
        return buildString {
            append("You are a realtime interpreter. Translate every user utterance into ")
            append(targetLanguage)
            append(". Speak only the translation, keep names and numbers intact, and do not answer questions except by translating the user's speech.")
            val extra = extraInstructions.trim()
            if (extra.isNotBlank()) {
                append('\n')
                append(extra)
            }
        }
    }

    private fun fail(message: String) {
        val redacted = SecretRedactor.redact(message)
        update {
            it.copy(
                busy = false,
                recording = false,
                status = "Error",
                statusDetail = "The current action could not continue.",
                lastError = redacted
            )
        }
        addEvent("Error: $redacted")
    }

    private fun postFailure(
        throwable: Throwable,
        transform: (VoiceUiState) -> VoiceUiState = { it.copy(busy = false) }
    ) {
        val message = SecretRedactor.redact(throwable.message ?: throwable.toString())
        mainHandler.post {
            update {
                transform(
                    it.copy(
                        status = "Error",
                        statusDetail = "The current action failed.",
                        lastError = message
                    )
                )
            }
            addEvent("Error: $message")
        }
    }

    private fun postUpdate(transform: (VoiceUiState) -> VoiceUiState) {
        mainHandler.post { update(transform) }
    }

    private fun postEvent(message: String) {
        mainHandler.post { addEvent(message) }
    }

    private fun update(transform: (VoiceUiState) -> VoiceUiState) {
        state = transform(state)
        observer?.invoke(state)
    }

    private fun addEvent(message: String) {
        val nextEvents = (listOf("${timeLabel()}  ${SecretRedactor.redact(message)}") + state.events).take(10)
        update { it.copy(events = nextEvents) }
    }

    private fun timeLabel(): String {
        val now = System.currentTimeMillis() / 1000L
        val seconds = now % 60L
        val minutes = (now / 60L) % 60L
        val hours = (now / 3600L) % 24L
        return String.format(Locale.US, "%02d:%02d:%02d", hours, minutes, seconds)
    }

    private fun modeDetail(mode: VoiceMode): String {
        return when (mode) {
            VoiceMode.TTS -> "Text to speech generation and local playback."
            VoiceMode.ASR -> "Record a short microphone sample and transcribe it."
            VoiceMode.REALTIME_CHAT -> "Live voice assistant through Rust oboe Realtime."
            VoiceMode.REALTIME_TRANSLATE -> "Live microphone translation through Rust oboe Realtime."
        }
    }

    private fun modeResultTitle(mode: VoiceMode): String {
        return when (mode) {
            VoiceMode.TTS -> "TTS output"
            VoiceMode.ASR -> "ASR transcript"
            VoiceMode.REALTIME_CHAT -> "Realtime transcript"
            VoiceMode.REALTIME_TRANSLATE -> "Realtime translation"
        }
    }

    private fun statusDetail(status: String): String {
        return when (status) {
            "Listening" -> "Listening to microphone input."
            "Thinking" -> "Speech detected, waiting for model response."
            "Responding" -> "Assistant audio is streaming back."
            "Connecting" -> "Opening Realtime WebSocket."
            "Connected" -> "Session connected, speak to the device."
            "Error" -> "The last operation returned an error."
            "Stopped" -> "Idle."
            "Stopping" -> "Closing session."
            else -> status.ifBlank { "Idle." }
        }
    }

    override fun close() {
        mainHandler.removeCallbacks(pollRunnable)
        executor.execute { repository.close() }
        executor.shutdown()
    }

    companion object {
        private const val DEFAULT_TTS_MODEL = "gpt-4o-mini-tts"
        private const val DEFAULT_ASR_MODEL = "gpt-4o-transcribe"
        private const val DEFAULT_REALTIME_MODEL = "gpt-realtime"
        private const val DEFAULT_REALTIME_INSTRUCTIONS =
            "You are a concise realtime voice assistant. Reply in the user's language."
        private const val DEFAULT_VOICE = "alloy"
        private const val DEFAULT_TARGET_LANGUAGE = "Chinese"
    }
}
