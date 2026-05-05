# Rust Android Device Smoke Runbook

Use this runbook before publishing or tagging a Rust Oboe alpha release.

## Preconditions

- Android SDK platform tools are installed.
- Android NDK is installed.
- At least one emulator or physical Android device is visible from `adb devices`.
- The repository-local Rust Android targets are installed for the ABIs being built.

## Build Native Libraries

```sh
ANDROID_NDK=/path/to/Android/Sdk/ndk/<version> tools/build-rust-android.sh
```

The script detects the NDK host prebuilt directory for Linux, macOS, and
Windows-like bash environments. If detection does not match your installed NDK,
set `ANDROID_NDK_HOST_TAG`, for example `ANDROID_NDK_HOST_TAG=linux-x86_64`.
Set `CARGO_TARGET_DIR` when build artifacts should be written outside
`rust/target`.

Expected output includes release builds for these Android ABIs:

```text
aarch64-linux-android
armv7-linux-androideabi
i686-linux-android
x86_64-linux-android
```

## Build Smoke APK

```sh
cd android/oboe-wrapper
ANDROID_USER_HOME=$HOME/.android \
JAVA_HOME=/path/to/jdk-17 \
PATH=/path/to/jdk-17/bin:$PATH \
./gradlew :oboe-smoke-app:assembleDebug --console=plain --no-daemon
```

Expected:

```text
BUILD SUCCESSFUL
```

## Install And Launch

```sh
adb devices
adb -s <device-serial> install -r android/oboe-wrapper/oboe-smoke-app/build/outputs/apk/debug/oboe-smoke-app-debug.apk
adb -s <device-serial> shell am start -n com.google.oboe.smoke/.MainActivity
```

Expected:

```text
Success
Starting: Intent { cmp=com.google.oboe.smoke/.MainActivity }
```

## Logcat Checks

```sh
adb -s <device-serial> logcat -d -t 500 | rg "FATAL EXCEPTION|UnsatisfiedLinkError|oboe|AAudio"
```

Release-blocking failures:

- `FATAL EXCEPTION`
- `UnsatisfiedLinkError`
- Native library missing for the installed ABI
- AAudio stream open failure in the smoke app
