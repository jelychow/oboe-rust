# Rust-Native Oboe

This repository has been reduced to the Rust-native Android audio path.

The legacy C++ implementation, C++ public headers, CMake/Prefab build scripts, sample apps, and C++ test runner have been removed. The supported implementation is now:

- `rust/oboe-core`: backend-neutral stream, builder, FIFO, format, resampler, callback, and extension state.
- `rust/oboe-android`: Android AAudio and OpenSL ES backend FFI.
- `rust/oboe-jni`: JNI handle layer exposed to Java.
- `android/oboe-wrapper`: Android Java wrapper and smoke tests.
- `tools/build-rust-android.ps1`: Android ABI build helper for `liboboe_jni.so`.

## Build and Test

```sh
cargo fmt --manifest-path rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rust/Cargo.toml --workspace --tests -- -D warnings
cargo test --manifest-path rust/Cargo.toml
```

To compile the Java wrapper without Gradle:

```powershell
javac.exe -Xlint:all -d build\javac-oboe-wrapper android\oboe-wrapper\oboe-wrapper\src\main\java\com\google\oboe\*.java
```

To build Android JNI libraries, provide an Android NDK path:

```powershell
.\tools\build-rust-android.ps1 -AndroidNdk C:\path\to\Android\Sdk\ndk\<version>
```
