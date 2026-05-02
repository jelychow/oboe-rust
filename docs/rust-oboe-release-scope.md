# Rust Oboe Alpha Release Scope

This repository contains a Rust-native Android audio path inspired by C++ Oboe.
The `0.1.0-alpha.1` release is intentionally smaller than C++ Oboe and should
be described as experimental.

## Publishable Crates

| Crate | Published in alpha | Purpose |
| --- | --- | --- |
| `oboe-core` | Yes | Backend-neutral builder, lifecycle, format, FIFO, and capability metadata. |
| `oboe-android` | Yes | Android AAudio and OpenSL ES backend bindings. |
| `oboe-samples` | Yes | Host-testable sample audio engines. |
| `oboe-jni` | No | JNI bridge requires AAR and ABI packaging decisions. |

## Example JNI Crates

Demo-specific JNI crates live in `examples/rust` instead of the publishable
Rust workspace:

| Crate | Path | Purpose |
| --- | --- | --- |
| `oboe-samples-jni` | `examples/rust/oboe-samples-jni` | Android bridge for `android/oboe-wrapper/oboe-samples-app`. |

The old standalone `minimaloboe-rust-jni`, `minimaloboe-rust-app`, and
`openai-realtime-jni` demo bridge paths have been removed to keep the public API
surface focused. The preserved OpenAI demo now uses Kotlin/Ktor for Realtime
networking and the Android Oboe SDK wrapper for audio I/O.

## C++ Oboe Parity Snapshot

| Area | C++ Oboe | Rust alpha status |
| --- | --- | --- |
| Stream builder basics | Full builder with API, direction, sharing, performance, sample rate, channel count, format, callback sizes | Partial: API, direction, sharing, performance, sample rate, channel count, format, callback config |
| Android stream lifecycle | open, start, stop, pause, flush, release, close, wait-for-state-change | Partial: open, request start, request stop, close, state |
| Blocking I/O | read and write | Partial: AAudio read/write, OpenSL ES output enqueue |
| Data callbacks | Native callback thread support | Not supported in alpha |
| Error callbacks | Disconnect and stream error callbacks | Not supported in alpha |
| Routing callbacks | Device route updates | Stored in config only; no callback dispatch |
| Timestamp and latency | timestamp, latency calculation, xrun count | Not supported in alpha |
| Buffer tuning | capacity, size, burst, available frames | Not supported in alpha |
| Advanced builder fields | usage, content type, input preset, session, device id, capture policy, privacy, spatialization, attribution | Not supported in alpha |
| Full duplex helper | `FullDuplexStream` | Not supported in alpha |
| Latency tuner | `LatencyTuner` | Not supported in alpha |

## Release Rule

Do not describe the alpha crates as a drop-in C++ Oboe replacement. Describe
them as experimental Rust-native building blocks for Android audio.
