package com.example.openairustrealtime

import android.Manifest
import android.app.Activity
import android.content.pm.PackageManager
import android.graphics.Color
import android.graphics.Typeface
import android.graphics.drawable.GradientDrawable
import android.media.AudioManager
import android.os.Bundle
import android.text.InputType
import android.view.Gravity
import android.view.View
import android.view.ViewGroup
import android.widget.ArrayAdapter
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.Spinner
import android.widget.TextView
import android.widget.Toast
import com.example.openairustrealtime.core.data.ApiKeyFieldPolicy
import com.example.openairustrealtime.core.model.VoiceMode
import com.example.openairustrealtime.core.model.VoiceUiState
import com.example.openairustrealtime.core.util.AppLog
import com.example.openairustrealtime.feature.voice.VoiceStateHolder
import java.util.Locale

class MainActivity : Activity() {
    private lateinit var stateHolder: VoiceStateHolder
    private lateinit var apiKeyInput: EditText
    private lateinit var keyStateView: TextView
    private lateinit var micPermissionView: TextView
    private lateinit var statusBadgeView: TextView
    private lateinit var statusDetailView: TextView
    private lateinit var signalView: RealtimeSignalView
    private lateinit var micChunksView: TextView
    private lateinit var micFramesView: TextView
    private lateinit var droppedMicView: TextView
    private lateinit var outputChunksView: TextView
    private lateinit var outputFramesView: TextView
    private lateinit var xrunView: TextView
    private lateinit var latencyView: TextView
    private lateinit var bufferView: TextView
    private lateinit var ttsModeButton: Button
    private lateinit var asrModeButton: Button
    private lateinit var realtimeModeButton: Button
    private lateinit var translateModeButton: Button
    private lateinit var ttsPanel: LinearLayout
    private lateinit var asrPanel: LinearLayout
    private lateinit var realtimePanel: LinearLayout
    private lateinit var translatePanel: LinearLayout
    private lateinit var ttsTextInput: EditText
    private lateinit var ttsModelInput: EditText
    private lateinit var ttsInstructionsInput: EditText
    private lateinit var ttsVoiceSpinner: Spinner
    private lateinit var runTtsButton: Button
    private lateinit var asrModelInput: EditText
    private lateinit var recordAsrButton: Button
    private lateinit var transcribeAsrButton: Button
    private lateinit var cancelAsrButton: Button
    private lateinit var realtimeChatModelInput: EditText
    private lateinit var realtimeChatInstructionsInput: EditText
    private lateinit var startRealtimeButton: Button
    private lateinit var stopRealtimeButton: Button
    private lateinit var realtimeModelInput: EditText
    private lateinit var targetLanguageInput: EditText
    private lateinit var realtimeInstructionsInput: EditText
    private lateinit var startTranslateButton: Button
    private lateinit var stopTranslateButton: Button
    private lateinit var requestMicButton: Button
    private lateinit var saveKeyButton: Button
    private lateinit var clearKeyButton: Button
    private lateinit var resultTitleView: TextView
    private lateinit var resultTextView: TextView
    private lateinit var eventFeedView: TextView
    private lateinit var errorView: TextView
    private var pendingMicAction: (() -> Unit)? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        volumeControlStream = AudioManager.STREAM_VOICE_CALL
        stateHolder = VoiceStateHolder(this)
        setContentView(createContentView())
        apiKeyInput.setText(stateHolder.restoredApiKeyInput())
        stateHolder.observe(::render)
        stateHolder.startPolling()
        requestMicPermissionIfNeeded()
    }

    override fun onStart() {
        super.onStart()
        AppLog.d(TAG, "Activity onStart ${stateHolder.debugState()}")
    }

    override fun onResume() {
        super.onResume()
        AppLog.d(TAG, "Activity onResume ${stateHolder.debugState()}")
    }

    override fun onPause() {
        AppLog.d(TAG, "Activity onPause ${stateHolder.debugState()}")
        super.onPause()
    }

    override fun onStop() {
        AppLog.i(TAG, "Activity onStop; stopping realtime if active. ${stateHolder.debugState()}")
        stateHolder.stopRealtime("activity.onStop")
        stateHolder.cancelAsrRecording()
        super.onStop()
    }

    override fun onDestroy() {
        AppLog.d(TAG, "Activity onDestroy ${stateHolder.debugState()}")
        stateHolder.close()
        super.onDestroy()
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode != REQUEST_RECORD_AUDIO) return
        val granted = grantResults.firstOrNull() == PackageManager.PERMISSION_GRANTED
        stateHolder.setMicPermission(granted)
        if (granted) {
            Toast.makeText(this, "Microphone permission granted.", Toast.LENGTH_SHORT).show()
            pendingMicAction?.invoke()
        } else {
            Toast.makeText(this, "Microphone permission is required for voice input.", Toast.LENGTH_LONG).show()
        }
        pendingMicAction = null
    }

    private fun createContentView(): View {
        val scrollView = ScrollView(this).apply {
            isFillViewport = true
            setBackgroundColor(COLOR_BACKGROUND)
        }

        val root = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(dp(20), dp(18), dp(20), dp(28))
        }
        scrollView.addView(root, ViewGroup.LayoutParams(match(), wrap()))

        root.addView(header(), fullWidthWithBottomMargin(dp(14)))
        root.addView(livePanel(), fullWidthWithBottomMargin(dp(14)))
        root.addView(metricsPanel(), fullWidthWithBottomMargin(dp(14)))
        root.addView(setupPanel(), fullWidthWithBottomMargin(dp(14)))
        root.addView(modePanel(), fullWidthWithBottomMargin(dp(14)))
        root.addView(errorPanel(), fullWidthWithBottomMargin(dp(14)))
        root.addView(resultPanel(), fullWidthWithBottomMargin(dp(14)))
        root.addView(eventPanel(), fullWidth())
        return scrollView
    }

    private fun header(): View {
        return LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER_HORIZONTAL
            setPadding(0, dp(8), 0, dp(8))
            addView(TextView(context).apply {
                text = "Voice"
                setTextColor(COLOR_TEXT)
                textSize = 34f
                typeface = Typeface.DEFAULT_BOLD
            })
            addView(TextView(context).apply {
                text = "TTS, ASR, realtime chat, and translate with Ktor and the Oboe SDK"
                setTextColor(COLOR_MUTED)
                textSize = 14f
                gravity = Gravity.CENTER
            }, fullWidth())
        }
    }

    private fun livePanel(): View {
        return panel().apply {
            gravity = Gravity.CENTER_HORIZONTAL
            statusBadgeView = TextView(context).apply {
                gravity = Gravity.CENTER
                setTextColor(COLOR_BACKGROUND)
                textSize = 13f
                typeface = Typeface.DEFAULT_BOLD
                setPadding(dp(14), dp(7), dp(14), dp(7))
            }
            addView(statusBadgeView, wrapContentCentered())

            statusDetailView = mutedText(14f).apply {
                gravity = Gravity.CENTER
                setPadding(0, dp(12), 0, dp(12))
            }
            addView(statusDetailView, fullWidth())

            signalView = RealtimeSignalView(context)
            addView(signalView, LinearLayout.LayoutParams(match(), dp(148)))
        }
    }

    private fun metricsPanel(): View {
        return LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            val firstRow = metricRow()
            val secondRow = metricRow()
            micChunksView = addMetric(firstRow, "Mic chunks")
            micFramesView = addMetric(firstRow, "Mic frames")
            droppedMicView = addMetric(firstRow, "Dropped mic")
            outputChunksView = addMetric(firstRow, "Audio chunks")
            outputFramesView = addMetric(secondRow, "Audio frames")
            xrunView = addMetric(secondRow, "Xruns")
            latencyView = addMetric(secondRow, "Latency")
            bufferView = addMetric(secondRow, "Out buffer")
            addView(firstRow, fullWidth())
            addView(secondRow, fullWidthWithTopMargin(dp(6)))
        }
    }

    private fun setupPanel(): View {
        return panel().apply {
            addView(sectionTitle("Setup"), fullWidth())
            apiKeyInput = input("OpenAI API key", password = true)
            addView(apiKeyInput, fullWidthWithTopMargin(dp(12)))

            val keyControls = LinearLayout(context).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER
            }
            saveKeyButton = primaryButton("Save Key")
            saveKeyButton.setOnClickListener {
                stateHolder.saveKey(apiKeyInput.text.toString())
                apiKeyInput.setText(stateHolder.restoredApiKeyInput())
            }
            clearKeyButton = secondaryButton("Clear")
            clearKeyButton.setOnClickListener {
                stateHolder.clearKey()
                apiKeyInput.setText("")
            }
            keyControls.addView(saveKeyButton, weightedButtonLayout())
            keyControls.addView(clearKeyButton, weightedButtonLayout())
            addView(keyControls, fullWidthWithTopMargin(dp(10)))

            keyStateView = mutedText(13f).apply { setPadding(0, dp(12), 0, 0) }
            addView(keyStateView, fullWidth())

            micPermissionView = mutedText(13f).apply { setPadding(0, dp(8), 0, 0) }
            addView(micPermissionView, fullWidth())

            requestMicButton = secondaryButton("Request Mic")
            requestMicButton.setOnClickListener { requestMicPermission() }
            addView(requestMicButton, fullWidthWithTopMargin(dp(10)))
        }
    }

    private fun modePanel(): View {
        return LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            addView(modeButtons(), fullWidthWithBottomMargin(dp(10)))
            ttsPanel = ttsPanel()
            asrPanel = asrPanel()
            realtimePanel = realtimePanel()
            translatePanel = translatePanel()
            addView(ttsPanel, fullWidth())
            addView(asrPanel, fullWidth())
            addView(realtimePanel, fullWidth())
            addView(translatePanel, fullWidth())
        }
    }

    private fun modeButtons(): View {
        return LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            val firstRow = LinearLayout(context).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER
            }
            val secondRow = LinearLayout(context).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER
            }
            ttsModeButton = secondaryButton("TTS").apply {
                setOnClickListener { stateHolder.selectMode(VoiceMode.TTS) }
            }
            asrModeButton = secondaryButton("ASR").apply {
                setOnClickListener { stateHolder.selectMode(VoiceMode.ASR) }
            }
            realtimeModeButton = secondaryButton("Realtime").apply {
                setOnClickListener { stateHolder.selectMode(VoiceMode.REALTIME_CHAT) }
            }
            translateModeButton = secondaryButton("Translate").apply {
                setOnClickListener { stateHolder.selectMode(VoiceMode.REALTIME_TRANSLATE) }
            }
            firstRow.addView(ttsModeButton, weightedButtonLayout())
            firstRow.addView(asrModeButton, weightedButtonLayout())
            secondRow.addView(realtimeModeButton, weightedButtonLayout())
            secondRow.addView(translateModeButton, weightedButtonLayout())
            addView(firstRow, fullWidth())
            addView(secondRow, fullWidthWithTopMargin(dp(8)))
        }
    }

    private fun ttsPanel(): LinearLayout {
        return panel().apply {
            addView(sectionTitle("Text to speech"), fullWidth())
            ttsTextInput = multiLineInput("Text to synthesize", minLines = 4).apply {
                setText("Hello from a Kotlin Android sample using OpenAI text to speech.")
            }
            addView(ttsTextInput, fullWidthWithTopMargin(dp(12)))

            ttsModelInput = input("TTS model", password = false).apply {
                setText("gpt-4o-mini-tts")
            }
            addView(ttsModelInput, fullWidthWithTopMargin(dp(10)))

            ttsVoiceSpinner = Spinner(context).apply {
                adapter = voiceAdapter()
                setSelection(TTS_VOICES.indexOf("alloy").coerceAtLeast(0))
                background = roundedStroke(COLOR_PANEL_LIGHT, COLOR_LINE, 1, 8)
                setPadding(dp(8), 0, dp(8), 0)
            }
            addView(ttsVoiceSpinner, LinearLayout.LayoutParams(match(), dp(48)).withTop(dp(10)))

            ttsInstructionsInput = multiLineInput("Voice instructions", minLines = 2).apply {
                setText("Speak naturally, warm, and concise.")
            }
            addView(ttsInstructionsInput, fullWidthWithTopMargin(dp(10)))

            runTtsButton = primaryButton("Synthesize and Play")
            runTtsButton.setOnClickListener {
                stateHolder.runTts(
                    apiKey = apiKeyInput.text.toString(),
                    text = ttsTextInput.text.toString(),
                    model = ttsModelInput.text.toString(),
                    voice = ttsVoiceSpinner.selectedItem?.toString().orEmpty(),
                    instructions = ttsInstructionsInput.text.toString()
                )
            }
            addView(runTtsButton, fullWidthWithTopMargin(dp(12)))
        }
    }

    private fun asrPanel(): LinearLayout {
        return panel().apply {
            addView(sectionTitle("Speech to text"), fullWidth())
            asrModelInput = input("ASR model", password = false).apply {
                setText("gpt-4o-transcribe")
            }
            addView(asrModelInput, fullWidthWithTopMargin(dp(12)))

            val controls = LinearLayout(context).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER
            }
            recordAsrButton = primaryButton("Record")
            recordAsrButton.setOnClickListener {
                requireMicPermission { stateHolder.startAsrRecording() }
            }
            transcribeAsrButton = secondaryButton("Stop and Transcribe")
            transcribeAsrButton.setOnClickListener {
                stateHolder.stopAsrAndTranscribe(
                    apiKey = apiKeyInput.text.toString(),
                    model = asrModelInput.text.toString()
                )
            }
            cancelAsrButton = secondaryButton("Cancel")
            cancelAsrButton.setOnClickListener { stateHolder.cancelAsrRecording() }
            controls.addView(recordAsrButton, weightedButtonLayout())
            controls.addView(transcribeAsrButton, weightedButtonLayout())
            controls.addView(cancelAsrButton, weightedButtonLayout())
            addView(controls, fullWidthWithTopMargin(dp(12)))
        }
    }

    private fun realtimePanel(): LinearLayout {
        return panel().apply {
            addView(sectionTitle("Realtime conversation"), fullWidth())
            realtimeChatModelInput = input("Realtime model", password = false).apply {
                setText("gpt-realtime")
            }
            addView(realtimeChatModelInput, fullWidthWithTopMargin(dp(12)))

            realtimeChatInstructionsInput = multiLineInput("Realtime instructions", minLines = 3).apply {
                setText("You are a concise realtime voice assistant. Reply in the user's language.")
            }
            addView(realtimeChatInstructionsInput, fullWidthWithTopMargin(dp(10)))

            val controls = LinearLayout(context).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER
            }
            startRealtimeButton = primaryButton("Start Realtime")
            startRealtimeButton.setOnClickListener {
                AppLog.i(TAG, "Start realtime chat clicked")
                requireMicPermission {
                    stateHolder.startRealtimeChat(
                        apiKey = apiKeyInput.text.toString(),
                        model = realtimeChatModelInput.text.toString(),
                        instructions = realtimeChatInstructionsInput.text.toString()
                    )
                }
            }
            stopRealtimeButton = secondaryButton("Stop")
            stopRealtimeButton.setOnClickListener {
                AppLog.i(TAG, "Stop realtime chat clicked")
                stateHolder.stopRealtime("chat.stopButton")
            }
            controls.addView(startRealtimeButton, weightedButtonLayout())
            controls.addView(stopRealtimeButton, weightedButtonLayout())
            addView(controls, fullWidthWithTopMargin(dp(12)))
        }
    }

    private fun translatePanel(): LinearLayout {
        return panel().apply {
            addView(sectionTitle("Realtime translate"), fullWidth())
            realtimeModelInput = input("Realtime model", password = false).apply {
                setText("gpt-realtime")
            }
            addView(realtimeModelInput, fullWidthWithTopMargin(dp(12)))

            targetLanguageInput = input("Target language", password = false).apply {
                setText("Chinese")
            }
            addView(targetLanguageInput, fullWidthWithTopMargin(dp(10)))

            realtimeInstructionsInput = multiLineInput("Extra translation instructions", minLines = 2).apply {
                setText("Preserve tone. Keep the translation concise.")
            }
            addView(realtimeInstructionsInput, fullWidthWithTopMargin(dp(10)))

            val controls = LinearLayout(context).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER
            }
            startTranslateButton = primaryButton("Start")
            startTranslateButton.setOnClickListener {
                AppLog.i(TAG, "Start realtime translate clicked")
                requireMicPermission {
                    stateHolder.startRealtimeTranslate(
                        apiKey = apiKeyInput.text.toString(),
                        model = realtimeModelInput.text.toString(),
                        targetLanguage = targetLanguageInput.text.toString(),
                        extraInstructions = realtimeInstructionsInput.text.toString()
                    )
                }
            }
            stopTranslateButton = secondaryButton("Stop")
            stopTranslateButton.setOnClickListener {
                AppLog.i(TAG, "Stop realtime translate clicked")
                stateHolder.stopRealtime("translate.stopButton")
            }
            controls.addView(startTranslateButton, weightedButtonLayout())
            controls.addView(stopTranslateButton, weightedButtonLayout())
            addView(controls, fullWidthWithTopMargin(dp(12)))
        }
    }

    private fun errorPanel(): View {
        errorView = TextView(this).apply {
            setTextColor(COLOR_ERROR)
            textSize = 13f
            setPadding(dp(14), dp(12), dp(14), dp(12))
            visibility = View.GONE
            background = roundedStroke(COLOR_PANEL_LIGHT, COLOR_ERROR, 1, 8)
        }
        return errorView
    }

    private fun resultPanel(): View {
        return panel().apply {
            resultTitleView = sectionTitle("Realtime output")
            addView(resultTitleView, fullWidth())
            resultTextView = TextView(context).apply {
                setTextColor(COLOR_TEXT)
                textSize = 16f
                setLineSpacing(dp(2).toFloat(), 1f)
                setTextIsSelectable(true)
                minLines = 6
                setPadding(0, dp(12), 0, 0)
            }
            addView(resultTextView, fullWidth())
        }
    }

    private fun eventPanel(): View {
        return panel().apply {
            addView(sectionTitle("Events"), fullWidth())
            eventFeedView = mutedText(13f).apply {
                typeface = Typeface.MONOSPACE
                setPadding(0, dp(12), 0, 0)
            }
            addView(eventFeedView, fullWidth())
        }
    }

    private fun render(state: VoiceUiState) {
        statusBadgeView.text = state.status.uppercase(Locale.US)
        statusBadgeView.background = rounded(statusColor(state.status), 99)
        statusDetailView.text = state.statusDetail
        keyStateView.text = if (state.savedKeyPresent) {
            "API key saved on this device; leave the field blank to reuse it"
        } else {
            "API key not saved"
        }
        apiKeyInput.hint = ApiKeyFieldPolicy.inputHint(state.savedKeyPresent)
        micPermissionView.text = if (state.micPermissionGranted) {
            "Microphone permission granted"
        } else {
            "Microphone permission required for ASR and realtime"
        }
        requestMicButton.text = if (state.micPermissionGranted) "Mic Granted" else "Request Mic"

        micChunksView.text = state.stats.inputChunks.toString()
        micFramesView.text = state.stats.inputFrames.toString()
        droppedMicView.text = state.stats.droppedInputFrames.toString()
        outputChunksView.text = state.stats.outputChunks.toString()
        outputFramesView.text = state.stats.outputFrames.toString()
        xrunView.text = "${state.stats.inputXRunCount}/${state.stats.outputXRunCount}"
        latencyView.text = String.format(Locale.US, "%.1f ms", state.stats.outputLatencyMillis)
        bufferView.text = bufferMetric(
            state.stats.outputBufferSizeFrames,
            state.stats.outputBufferCapacityFrames
        )
        val audioActive = state.realtimeRunning || state.recording || state.status == "Playing"
        signalView.setLive(audioActive && state.status != "Error")
        signalView.setLevels(state.micLevel, state.outputLevel)

        ttsPanel.visibility = if (state.selectedMode == VoiceMode.TTS) View.VISIBLE else View.GONE
        asrPanel.visibility = if (state.selectedMode == VoiceMode.ASR) View.VISIBLE else View.GONE
        realtimePanel.visibility =
            if (state.selectedMode == VoiceMode.REALTIME_CHAT) View.VISIBLE else View.GONE
        translatePanel.visibility =
            if (state.selectedMode == VoiceMode.REALTIME_TRANSLATE) View.VISIBLE else View.GONE
        updateModeButton(ttsModeButton, state.selectedMode == VoiceMode.TTS)
        updateModeButton(asrModeButton, state.selectedMode == VoiceMode.ASR)
        updateModeButton(realtimeModeButton, state.selectedMode == VoiceMode.REALTIME_CHAT)
        updateModeButton(translateModeButton, state.selectedMode == VoiceMode.REALTIME_TRANSLATE)

        resultTitleView.text = state.resultTitle
        resultTextView.text = state.resultText
        eventFeedView.text = state.events.joinToString(separator = "\n")
        errorView.visibility = if (state.lastError.isBlank()) View.GONE else View.VISIBLE
        errorView.text = "Last error: ${state.lastError}"

        val idle = !state.busy && !state.recording && !state.realtimeRunning
        requestMicButton.isEnabled = !state.micPermissionGranted
        saveKeyButton.isEnabled = idle
        clearKeyButton.isEnabled = idle
        runTtsButton.isEnabled = idle
        recordAsrButton.isEnabled = idle && state.micPermissionGranted
        transcribeAsrButton.isEnabled = state.recording
        cancelAsrButton.isEnabled = state.recording
        startRealtimeButton.isEnabled = idle && state.micPermissionGranted
        stopRealtimeButton.isEnabled =
            state.realtimeRunning || (state.busy && state.selectedMode == VoiceMode.REALTIME_CHAT)
        startTranslateButton.isEnabled = idle && state.micPermissionGranted
        stopTranslateButton.isEnabled =
            state.realtimeRunning || (state.busy && state.selectedMode == VoiceMode.REALTIME_TRANSLATE)

        listOf(
            requestMicButton,
            saveKeyButton,
            clearKeyButton,
            runTtsButton,
            recordAsrButton,
            transcribeAsrButton,
            cancelAsrButton,
            startRealtimeButton,
            stopRealtimeButton,
            startTranslateButton,
            stopTranslateButton
        ).forEach { button ->
            button.alpha = if (button.isEnabled) 1f else 0.45f
        }
    }

    private fun requestMicPermissionIfNeeded() {
        if (!hasRecordAudioPermission()) requestMicPermission()
    }

    private fun requireMicPermission(action: () -> Unit) {
        if (hasRecordAudioPermission()) {
            action()
        } else {
            pendingMicAction = action
            requestMicPermission()
        }
    }

    private fun requestMicPermission() {
        requestPermissions(arrayOf(Manifest.permission.RECORD_AUDIO), REQUEST_RECORD_AUDIO)
    }

    private fun hasRecordAudioPermission(): Boolean {
        return checkSelfPermission(Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED
    }

    private fun panel(): LinearLayout {
        return LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(dp(16), dp(16), dp(16), dp(16))
            background = roundedStroke(COLOR_PANEL, COLOR_LINE, 1, 8)
        }
    }

    private fun input(hint: String, password: Boolean): EditText {
        return EditText(this).apply {
            this.hint = hint
            setHintTextColor(COLOR_MUTED)
            setTextColor(COLOR_TEXT)
            textSize = 15f
            isSingleLine = true
            setPadding(dp(14), 0, dp(14), 0)
            background = roundedStroke(COLOR_PANEL_LIGHT, COLOR_LINE, 1, 8)
            inputType = if (password) {
                InputType.TYPE_CLASS_TEXT or
                    InputType.TYPE_TEXT_VARIATION_PASSWORD or
                    InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS
            } else {
                InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS
            }
        }
    }

    private fun multiLineInput(hint: String, minLines: Int): EditText {
        return input(hint, password = false).apply {
            isSingleLine = false
            this.minLines = minLines
            gravity = Gravity.TOP or Gravity.START
            setPadding(dp(14), dp(10), dp(14), dp(10))
            inputType = InputType.TYPE_CLASS_TEXT or
                InputType.TYPE_TEXT_FLAG_MULTI_LINE or
                InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS
        }
    }

    private fun sectionTitle(text: String): TextView {
        return TextView(this).apply {
            this.text = text
            setTextColor(COLOR_TEXT)
            textSize = 14f
            typeface = Typeface.DEFAULT_BOLD
        }
    }

    private fun mutedText(size: Float): TextView {
        return TextView(this).apply {
            setTextColor(COLOR_MUTED)
            textSize = size
        }
    }

    private fun addMetric(parent: LinearLayout, label: String): TextView {
        val box = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER
            setPadding(dp(8), dp(10), dp(8), dp(10))
            background = roundedStroke(COLOR_PANEL, COLOR_LINE, 1, 8)
        }
        val value = TextView(this).apply {
            text = "0"
            setTextColor(COLOR_TEXT)
            textSize = 17f
            typeface = Typeface.DEFAULT_BOLD
            gravity = Gravity.CENTER
        }
        box.addView(value, fullWidth())
        box.addView(mutedText(11f).apply {
            text = label
            gravity = Gravity.CENTER
        }, fullWidth())
        parent.addView(box, LinearLayout.LayoutParams(0, wrap(), 1f).withMargins(dp(3), 0, dp(3), 0))
        return value
    }

    private fun metricRow(): LinearLayout {
        return LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER
        }
    }

    private fun bufferMetric(sizeFrames: Int, capacityFrames: Int): String {
        return if (sizeFrames > 0 && capacityFrames > 0) {
            "$sizeFrames/$capacityFrames"
        } else {
            "-"
        }
    }

    private fun primaryButton(text: String): Button {
        return baseButton(text).apply {
            setTextColor(COLOR_BACKGROUND)
            background = rounded(COLOR_TEXT, 8)
        }
    }

    private fun secondaryButton(text: String): Button {
        return baseButton(text).apply {
            setTextColor(COLOR_TEXT)
            background = roundedStroke(COLOR_PANEL_LIGHT, COLOR_LINE, 1, 8)
        }
    }

    private fun baseButton(text: String): Button {
        return Button(this).apply {
            setAllCaps(false)
            this.text = text
            textSize = 14f
            minHeight = dp(44)
            setPadding(dp(8), 0, dp(8), 0)
        }
    }

    private fun updateModeButton(button: Button, selected: Boolean) {
        button.setTextColor(if (selected) COLOR_BACKGROUND else COLOR_TEXT)
        button.background = if (selected) {
            rounded(COLOR_ACCENT, 8)
        } else {
            roundedStroke(COLOR_PANEL_LIGHT, COLOR_LINE, 1, 8)
        }
    }

    private fun voiceAdapter(): ArrayAdapter<String> {
        return object : ArrayAdapter<String>(
            this,
            android.R.layout.simple_spinner_dropdown_item,
            TTS_VOICES
        ) {
            override fun getView(position: Int, convertView: View?, parent: ViewGroup): View {
                return super.getView(position, convertView, parent).apply {
                    if (this is TextView) {
                        setTextColor(COLOR_TEXT)
                        setBackgroundColor(COLOR_PANEL_LIGHT)
                    }
                }
            }

            override fun getDropDownView(position: Int, convertView: View?, parent: ViewGroup): View {
                return super.getDropDownView(position, convertView, parent).apply {
                    if (this is TextView) {
                        setTextColor(Color.BLACK)
                    }
                }
            }
        }
    }

    private fun statusColor(status: String): Int {
        return when (status) {
            "Error" -> COLOR_ERROR
            "Listening", "Recording", "Transcribed", "Playing" -> COLOR_ACCENT
            "Thinking", "Responding", "Connecting", "Synthesizing", "Transcribing", "Stopping" -> COLOR_WARN
            else -> COLOR_TEXT
        }
    }

    private fun rounded(color: Int, radiusDp: Int): GradientDrawable {
        return GradientDrawable().apply {
            setColor(color)
            cornerRadius = dp(radiusDp).toFloat()
        }
    }

    private fun roundedStroke(color: Int, stroke: Int, strokeDp: Int, radiusDp: Int): GradientDrawable {
        return rounded(color, radiusDp).apply {
            setStroke(dp(strokeDp), stroke)
        }
    }

    private fun fullWidth(): LinearLayout.LayoutParams = LinearLayout.LayoutParams(match(), wrap())

    private fun fullWidthWithTopMargin(top: Int): LinearLayout.LayoutParams = fullWidth().withTop(top)

    private fun fullWidthWithBottomMargin(bottom: Int): LinearLayout.LayoutParams {
        return fullWidth().withMargins(0, 0, 0, bottom)
    }

    private fun wrapContentCentered(): LinearLayout.LayoutParams {
        return LinearLayout.LayoutParams(wrap(), wrap()).apply { gravity = Gravity.CENTER_HORIZONTAL }
    }

    private fun weightedButtonLayout(): LinearLayout.LayoutParams {
        return LinearLayout.LayoutParams(0, wrap(), 1f).withMargins(dp(4), 0, dp(4), 0)
    }

    private fun LinearLayout.LayoutParams.withTop(top: Int): LinearLayout.LayoutParams {
        setMargins(0, top, 0, 0)
        return this
    }

    private fun LinearLayout.LayoutParams.withMargins(
        left: Int,
        top: Int,
        right: Int,
        bottom: Int
    ): LinearLayout.LayoutParams {
        setMargins(left, top, right, bottom)
        return this
    }

    private fun match(): Int = ViewGroup.LayoutParams.MATCH_PARENT

    private fun wrap(): Int = ViewGroup.LayoutParams.WRAP_CONTENT

    private fun dp(value: Int): Int = (value * resources.displayMetrics.density).toInt()

    companion object {
        private const val TAG = "MainActivity"
        private const val REQUEST_RECORD_AUDIO = 1001
        private val TTS_VOICES = listOf(
            "alloy",
            "ash",
            "ballad",
            "coral",
            "echo",
            "fable",
            "nova",
            "onyx",
            "sage",
            "shimmer",
            "verse",
            "marin",
            "cedar"
        )
        private val COLOR_BACKGROUND = Color.rgb(17, 17, 15)
        private val COLOR_PANEL = Color.rgb(30, 30, 27)
        private val COLOR_PANEL_LIGHT = Color.rgb(39, 38, 34)
        private val COLOR_TEXT = Color.rgb(245, 243, 237)
        private val COLOR_MUTED = Color.rgb(166, 162, 153)
        private val COLOR_LINE = Color.rgb(57, 55, 50)
        private val COLOR_ACCENT = Color.rgb(170, 219, 191)
        private val COLOR_WARN = Color.rgb(238, 204, 126)
        private val COLOR_ERROR = Color.rgb(255, 176, 165)
    }
}
