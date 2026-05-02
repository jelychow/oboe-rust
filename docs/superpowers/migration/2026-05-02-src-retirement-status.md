# Legacy C++ Removal Status

The repository no longer keeps the legacy C++ implementation as a reference or build target.

Removed paths:

- `src`
- `include`
- `apps`
- `samples`
- `tests`
- `prefab`
- root `CMakeLists.txt`
- `rust/oboe_rust_core`

Remaining supported paths:

- `rust/oboe-core`
- `rust/oboe-android`
- `rust/oboe-jni`
- `android/oboe-wrapper`
- `tools/build-rust-android.ps1`

New functionality should target the Rust workspace first, then expose Java/JNI wrapper APIs as needed.
