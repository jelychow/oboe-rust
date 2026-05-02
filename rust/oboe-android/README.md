# oboe-android

Experimental Rust-native Android audio backends for Oboe-style streams.

Supported in `0.1.0-alpha.1`:

- AAudio stream open/start/stop/close.
- AAudio blocking `f32` read/write.
- AAudio `f32` and `i16` conversion paths.
- OpenSL ES blocking output enqueue for `f32`/`i16` conversion.

Known alpha limitations:

- OpenSL ES input is not a real recorder path.
- Realtime callbacks are not yet driven from native audio callback threads.
- Timestamp, xrun, latency, buffer tuning, pause, flush, and release APIs are not yet exposed.
- Android ABI builds require the NDK and should be verified on physical devices.

See `docs/rust-oboe-release-scope.md` in the repository for the full support matrix.
