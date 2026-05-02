# oboe-core

Backend-neutral Rust types for an experimental Rust-native Oboe audio API.

This crate is an alpha building block. It does not claim full API parity with
the C++ Oboe headers. The supported surface is intentionally small:

- Stream builder defaults and validation.
- Stream lifecycle state used by backend implementations.
- PCM format helpers.
- FIFO and small sample-rate interpolation helpers.
- Capability status metadata for the Rust-native release lane.

Use `oboe-android` when you need Android AAudio or OpenSL ES access.

## Release Status

Version `0.1.0-alpha.1` is suitable for experiments and internal validation.
Do not treat the API as stable until the repository publishes a non-alpha
version and removes the alpha scope warning from `docs/rust-oboe-release-scope.md`.
