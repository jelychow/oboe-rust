# Rust Oboe Alpha Release Checklist

Use this checklist for `0.1.0-alpha.1`.

## Required Checks

- `cargo fmt --manifest-path rust/Cargo.toml -p oboe-core -p oboe-android -p oboe-samples --check`
- `cargo clippy --manifest-path rust/Cargo.toml -p oboe-core -p oboe-android -p oboe-samples --tests -- -D warnings`
- `cargo test --manifest-path rust/Cargo.toml -p oboe-core -p oboe-android -p oboe-samples`
- `cargo publish --dry-run --manifest-path rust/Cargo.toml -p oboe-core --allow-dirty`
- `cargo package --manifest-path rust/Cargo.toml -p oboe-android --allow-dirty --list`
- `cargo package --manifest-path rust/Cargo.toml -p oboe-samples --allow-dirty --list`
- `CHECK_ANDROID_ABI=1 tools/check-rust-release.sh`
- Follow `docs/rust-android-device-smoke.md` on an emulator or physical device.

After `oboe-core` is published and visible in the crates.io index, run:

- `VERIFY_PUBLISHED_DEPS=1 tools/check-rust-release.sh`

## Publish Order

1. `oboe-core`
2. `oboe-android`
3. `oboe-samples`

## Release Notes

Use this release note text:

```text
Rust Oboe 0.1.0-alpha.1 introduces experimental Rust-native Android audio building blocks.

Published crates:
- oboe-core: backend-neutral stream builder, lifecycle, format, FIFO, and capability metadata.
- oboe-android: experimental AAudio/OpenSL ES backend access.
- oboe-samples: host-testable Rust sample engines.

This alpha is not a drop-in replacement for C++ Oboe. Callback dispatch, advanced builder fields, timestamp/latency/xrun diagnostics, buffer tuning, full-duplex helpers, and AAR/JNI artifact distribution remain outside this release.
```
