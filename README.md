# oboe-rust

This repository has been reduced to the Rust-native Android audio path.

The legacy C++ implementation, C++ public headers, CMake/Prefab build scripts, sample apps, and C++ test runner have been removed. The supported implementation is now:

- `rust/oboe-core`: backend-neutral stream, builder, FIFO, format, resampler, callback, and extension state.
- `rust/oboe-android`: Android AAudio and OpenSL ES backend FFI.
- `rust/oboe-jni`: JNI handle layer exposed to Java.
- `android/oboe-wrapper`: Android Java wrapper and smoke tests.
- `tools/build-rust-android.ps1`: Android ABI build helper for `liboboe_jni.so`.

## Release Scope

The Rust crates are currently alpha-quality. Before publishing or consuming them
as library dependencies, read `docs/rust-oboe-release-scope.md`. The alpha
release is not a drop-in replacement for the C++ Oboe API.

## Build and Test

```sh
cargo fmt --manifest-path rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rust/Cargo.toml --workspace --tests -- -D warnings
cargo test --manifest-path rust/Cargo.toml
```

## Rust Alpha Release Check

Run the release gate before publishing Rust crates:

```sh
tools/check-rust-release.sh
```

Include Android target checks for the publishable crates when the Rust Android
targets are installed:

```sh
CHECK_ANDROID_ABI=1 tools/check-rust-release.sh
```

For the first crates.io release, `oboe-core` must be published and visible in
the registry before dependent crate dry-runs can resolve it. After `oboe-core`
is indexed, run the dependent dry-runs:

```sh
VERIFY_PUBLISHED_DEPS=1 tools/check-rust-release.sh
```

JNI `.so` builds for Android sample apps are smoke checks rather than crates.io
publish gates. Use `docs/rust-android-device-smoke.md` for that path.

## Android Gradle Sync

Open the repository root in Android Studio. The root Gradle project exposes:

- `:oboe-wrapper`: Android library module for Java/JNI consumers.
- `:oboe-smoke-app`: installable smoke app using `implementation project(':oboe-wrapper')`.

To compile the Java wrapper without Gradle:

```powershell
javac.exe -Xlint:all -d build\javac-oboe-wrapper android\oboe-wrapper\oboe-wrapper\src\main\java\com\google\oboe\*.java
```

To build Android JNI libraries, provide an Android NDK path. The Rust AAudio backend
links against `libaaudio`, so Android API 26 is the default native build baseline.

```powershell
.\tools\build-rust-android.ps1 -AndroidNdk C:\path\to\Android\Sdk\ndk\<version>
```

To build a signed smoke-test APK without Gradle:

```powershell
.\tools\build-smoke-apk.ps1 -AndroidSdk C:\path\to\Android\Sdk
```

To install it on a connected device or emulator:

```powershell
.\tools\build-smoke-apk.ps1 -AndroidSdk C:\path\to\Android\Sdk -Install
```
