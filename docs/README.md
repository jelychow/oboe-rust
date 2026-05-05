# oboe-rust Docs

The active repository surface is Rust plus the Android Java/JNI wrapper.

## Active Paths

- `../rust/oboe-core`: core stream, builder, FIFO, format, resampler, callback, and extension state.
- `../rust/oboe-android`: Android AAudio and OpenSL ES backend bindings.
- `../rust/oboe-jni`: JNI entry points exposed to Java.
- `../android/oboe-wrapper`: Java wrapper project and smoke tests.
- `../tools/build-rust-android.ps1`: Android ABI build helper.
- `../tools/build-smoke-apk.ps1`: signed smoke APK build and optional install helper.
- `rust-oboe-release-scope.md`: alpha release boundaries and C++ Oboe parity snapshot.
- `rust-android-device-smoke.md`: Android ABI, APK, install, and logcat smoke runbook.

Open the repository root in Android Studio for Gradle sync. The root project maps
`:oboe-wrapper` to the Java/JNI library and `:oboe-smoke-app` to a minimal app
that depends on the wrapper.

Legacy C++ headers, implementation, sample apps, CMake/Prefab build files, Doxygen workflow, and C++ test runner were removed during the Rust-native migration.
