# Rust-Native Oboe Docs

The active repository surface is Rust plus the Android Java/JNI wrapper.

## Active Paths

- `../rust/oboe-core`: core stream, builder, FIFO, format, resampler, callback, and extension state.
- `../rust/oboe-android`: Android AAudio and OpenSL ES backend bindings.
- `../rust/oboe-jni`: JNI entry points exposed to Java.
- `../android/oboe-wrapper`: Java wrapper project and smoke tests.
- `../tools/build-rust-android.ps1`: Android ABI build helper.
- `../tools/build-smoke-apk.ps1`: signed smoke APK build and optional install helper.

Legacy C++ headers, implementation, sample apps, CMake/Prefab build files, Doxygen workflow, and C++ test runner were removed during the Rust-native migration.
