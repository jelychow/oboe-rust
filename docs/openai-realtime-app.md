# OpenAI Voice Kotlin/Ktor Android App

This sample app is at `android/oboe-wrapper/openai-realtime-app`.

It uses:

- Oboe SDK audio path: `implementation project(':oboe-wrapper')` with
  `com.google.oboe.AudioStream` input and output at 24 kHz mono
- Ktor OpenAI Realtime WebSocket:
  `wss://api.openai.com/v1/realtime?model=gpt-realtime`
- OpenAI Audio REST endpoints for TTS and ASR
- Kotlin Android UI with a Now in Android inspired split between model,
  network, data, feature state, and Activity code
- Android UI: saved API key, TTS, ASR, realtime chat, realtime translate, status badge,
  realtime signal visualization, audio counters, transcript/result panel, event
  feed, and clear-key controls

The API key is entered at runtime and saved in the app's private
`SharedPreferences` so repeated launches do not require retyping it. It is not
written to the repository. This is sample convenience storage, not production
secret storage; use Android Keystore-backed storage before shipping this app to
end users.

## Build Oboe SDK Native Library

The app does not ship its own Realtime JNI library. It consumes the Oboe SDK
module, so only `:oboe-wrapper` needs `liboboe_jni.so` built for Android ABIs.
On WSL or Linux:

```bash
ANDROID_NDK=/path/to/Android/Sdk/ndk/<version> \
RUST_ANDROID_LIBRARIES=oboe-jni \
tools/build-rust-android.sh
```

The full Rust Android build helper also builds the sample launcher JNI crate,
but it no longer builds a Realtime app JNI library.

## Build APK

After `android/oboe-wrapper/oboe-wrapper/src/main/jniLibs` contains
`liboboe_jni.so`, build the debug APK with Gradle:

```bash
cd android/oboe-wrapper
JAVA_HOME=/path/to/jdk-17 \
PATH=/path/to/jdk-17/bin:$PATH \
./gradlew :openai-realtime-app:assembleDebug --console=plain --no-daemon
```

Output:

```text
android/oboe-wrapper/openai-realtime-app/build/outputs/apk/debug/openai-realtime-app-debug.apk
```

The legacy PowerShell helper now delegates to Gradle when Kotlin sources are
present:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass `
  -File tools/build-openai-realtime-apk.ps1 `
  -AndroidSdk 'F:\Android\android-sdk' `
  -NdkVersion '29.0.14206865' `
  -SkipRustBuild
```

Output:

```text
build/openai-realtime-apk/openai-realtime-debug.apk
```

If Windows Gradle fails with `Unable to establish loopback connection`, run the
WSL Gradle command above with the repo-local Linux JDK instead.

## Run

Install and launch:

```bash
/mnt/f/Android/android-sdk/platform-tools/adb.exe install -r android/oboe-wrapper/openai-realtime-app/build/outputs/apk/debug/openai-realtime-app-debug.apk
/mnt/f/Android/android-sdk/platform-tools/adb.exe shell am start -n com.example.openairustrealtime/.MainActivity
```

In the app:

1. Enter an OpenAI API key and tap Save. The app also saves the key when a
   voice action starts.
2. Allow microphone permission when the app opens, or tap Request Mic.
3. Use TTS:
   - Select TTS.
   - Enter text, keep `gpt-4o-mini-tts`, choose a voice, and tap Synthesize and
     Play.
   - The app requests raw 24 kHz signed 16-bit PCM and plays it through the
     `com.google.oboe` SDK output stream.
4. Use ASR:
   - Select ASR.
   - Keep `gpt-4o-transcribe`, tap Record, speak briefly, then tap Stop and
     Transcribe.
   - Recording is captured through the `com.google.oboe` SDK input stream and
     written as a WAV file for the transcription request.
5. Use Realtime:
   - Select Realtime.
   - Keep `gpt-realtime`, adjust assistant instructions, and tap Start
     Realtime.
   - This is the original realtime voice assistant flow, not translation-only.
6. Use Realtime Translate:
   - Select Translate.
   - Keep `gpt-realtime`, choose a target language, and tap Start.
   - Speak to the device; translated assistant audio and transcript/status
     updates appear live.
7. Tap Clear to remove the locally saved key.

During a live session, `Mic sent` should increase when microphone audio is being
captured through the Oboe SDK and sent to Realtime. `Output played` should
increase when assistant audio chunks are received and written through the Oboe
SDK output stream.

The realtime screen mirrors those counters as live metric tiles and animates the
mic/assistant signal view from chunk deltas. The event feed records status
changes, mic chunk updates, transcript updates, and API errors while the session
is running.

The app configures both input and output PCM audio as 24 kHz mono. The Realtime
session update includes `session.audio.input.format.rate` and
`session.audio.output.format.rate` set to `24000`.

## Debugging Audio

Realtime networking now runs in Kotlin through Ktor. Use Android and app logs
for network/session issues:

```bash
/mnt/f/Android/android-sdk/platform-tools/adb.exe logcat | grep openairustrealtime
```

Expected healthy flow:

```text
Status: Connecting
Status: Connected
Mic +1 chunks
Status: Listening
Status: Responding
Audio +1 chunks
Transcript updated
```

If Android's `AAudio` logs show both input and output streams opening with
`AAUDIO_OK`, then microphone and playback devices are available. If `Mic sent`
increases but `Output played` stays at zero, inspect the visible error panel and
event feed first; the app is sending mic audio but is not receiving playable
assistant audio deltas.

Realtime server errors and UI error text redact OpenAI keys as `sk-***`.

## Kotlin Structure

The sample intentionally keeps one Android module but separates responsibilities:

- `core.model`: immutable state and request models.
- `core.network`: OpenAI speech/transcription HTTP calls, Ktor Realtime
  WebSocket session, and Realtime event protocol codec.
- `core.audio`: Oboe SDK PCM playback, WAV recording, and realtime audio pump.
- `core.data`: API key storage and repository orchestration.
- `feature.voice`: UI state holder and workflow orchestration.
- root package: `MainActivity` and `RealtimeSignalView`.

## Notes

- This is a local development sample. Passing a long-lived API key directly to a
  mobile app is not appropriate for production distribution.
- The saved key is app-private storage, not hardware-backed encryption.
- For production, use a backend to mint short-lived Realtime client secrets and
  pass only those ephemeral credentials to the app.
- WebRTC is generally more robust for mobile media transport, but this sample
  intentionally uses WebSocket so the OpenAI session bridge stays explicit and
  the Android audio path is exercised through the published Oboe SDK API.
