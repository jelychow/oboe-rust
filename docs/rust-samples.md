# Rust Sample Mapping

The upstream Android Oboe sample tree is represented by the `rust/oboe-samples`
crate. The Rust versions focus on the audio logic and state
machines so they can run under normal Cargo tests without an Android runtime.

| Upstream sample | Rust module | Runnable example |
| --- | --- | --- |
| `hello-oboe` | `oboe_samples::hello_oboe` | `cargo run --manifest-path rust/Cargo.toml -p oboe-samples --example hello_oboe` |
| `minimaloboe` | `oboe_samples::minimal_oboe` | `cargo run --manifest-path rust/Cargo.toml -p oboe-samples --example minimal_oboe` |
| `LiveEffect` | `oboe_samples::live_effect` | `cargo run --manifest-path rust/Cargo.toml -p oboe-samples --example live_effect` |
| `MegaDrone` | `oboe_samples::mega_drone` | `cargo run --manifest-path rust/Cargo.toml -p oboe-samples --example mega_drone` |
| `SoundBoard` | `oboe_samples::sound_board` | `cargo run --manifest-path rust/Cargo.toml -p oboe-samples --example sound_board` |
| `audio-device` | `oboe_samples::audio_device` | `cargo run --manifest-path rust/Cargo.toml -p oboe-samples --example audio_device` |
| `drumthumper` | `oboe_samples::drumthumper` | `cargo run --manifest-path rust/Cargo.toml -p oboe-samples --example drumthumper` |
| `powerplay` | `oboe_samples::powerplay` | `cargo run --manifest-path rust/Cargo.toml -p oboe-samples --example powerplay` |
| `RhythmGame` | `oboe_samples::rhythm_game` | `cargo run --manifest-path rust/Cargo.toml -p oboe-samples --example rhythm_game` |
| `iolib` | `oboe_samples::iolib` | `cargo run --manifest-path rust/Cargo.toml -p oboe-samples --example iolib` |
| `parselib` | `oboe_samples::parselib` | `cargo run --manifest-path rust/Cargo.toml -p oboe-samples --example parselib` |
| `shared` | `oboe_samples::shared` | `cargo run --manifest-path rust/Cargo.toml -p oboe-samples --example shared` |
| `debug-utils` | `oboe_samples::debug_utils` | `cargo run --manifest-path rust/Cargo.toml -p oboe-samples --example debug_utils` |

The Rust crate does not clone every upstream Java/Kotlin UI project one-for-one.
Instead, Android device execution is provided by a single launcher app that runs
the Rust version of each sample.

## Android Device App

`android/oboe-wrapper/oboe-samples-app` is a single launcher APK that runs every
Rust sample on an Android device. Each button calls `oboe-samples-jni`, which
renders the selected Rust sample and writes the generated float PCM through the
Rust Android backend.

Build and install from PowerShell:

```powershell
.\tools\build-samples-apk.ps1 -AndroidSdk C:\path\to\Android\Sdk -Install
```

The app defaults to AAudio and also exposes an OpenSL ES selector for devices
where that backend is available.

## Complete MinimalOboe Port

`android/oboe-wrapper/minimaloboe-rust-app` is a fuller Rust port of upstream
`samples/minimaloboe`:

- Java package: `com.example.minimaloboe`
- Native library: `libminimaloboe_rust.so`
- UI behavior: Start Audio, Stop Audio, and current status text
- Native behavior: static Rust player, low-latency float stereo output stream,
  background audio writer, white-noise generation, stop-on-background lifecycle

Build and install:

```powershell
.\tools\build-minimaloboe-rust-apk.ps1 -AndroidSdk C:\path\to\Android\Sdk -Install
```

The APK is also available after build at
`build\minimaloboe-rust-apk\minimaloboe-rust-debug.apk`.
