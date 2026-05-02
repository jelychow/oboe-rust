# Full Rust Oboe Rewrite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move liboboe functionality to Rust while keeping the existing C++ public API and existing tests as compatibility gates.

**Architecture:** Rust owns all implementation state and behavior through a staticlib with a stable C ABI. C++ headers and classes remain as source-compatible adapters until downstream users can migrate. Android platform calls stay behind Rust `extern "C"` bindings to AAudio, OpenSL ES, libc, and Android system libraries.

**Tech Stack:** Rust 2021 `staticlib`, C ABI bridge, existing CMake build, existing GoogleTest Android runner, existing C++ headers.

---

## File Structure

- `rust/oboe_rust_core/src/lib.rs`: crate root and exported C ABI.
- `rust/oboe_rust_core/src/definitions.rs`: Rust enum/value mirror of `include/oboe/Definitions.h`.
- `rust/oboe_rust_core/src/utilities.rs`: format, text, property, clock, and conversion utilities.
- `rust/oboe_rust_core/src/fifo.rs`: FIFO controller and buffer operations.
- `rust/oboe_rust_core/src/flowgraph.rs`: flowgraph nodes, ports, converters, sinks, and sources.
- `rust/oboe_rust_core/src/resampler.rs`: linear, polyphase, sinc, and multichannel resamplers.
- `rust/oboe_rust_core/src/stream.rs`: stream base state, builder settings, close/start/stop/pause/flush orchestration.
- `rust/oboe_rust_core/src/backend/aaudio.rs`: AAudio dynamic loader and stream backend.
- `rust/oboe_rust_core/src/backend/opensles.rs`: OpenSL ES engine, mixer, buffered stream, input, and output backend.
- `rust/oboe_rust_core/src/callback.rs`: callback trampoline and error callback handling.
- `src/rust/oboe_rust_core.h`: C ABI declarations used by C++ adapters.
- `src/**/*.cpp`: reduced to API-compatible adapters that forward to Rust.
- `tests/*.cpp`: unchanged compatibility tests; add tests only when existing coverage is missing.

## Task 1: Build And Test Contract

**Files:**
- Modify: `CMakeLists.txt`
- Modify: `tests/run_tests.sh`
- Create: `rust/oboe_rust_core/.cargo/config.toml`

- [ ] Keep `OBOE_USE_RUST_CORE=ON` as the test path used by `tests/run_tests.sh`.
- [ ] Add Android target linker config for `aarch64-linux-android`, `armv7-linux-androideabi`, `i686-linux-android`, and `x86_64-linux-android`.
- [ ] Run: `tr -d '\r' < tests/run_tests.sh | PATH="$HOME/.cargo/bin:$PATH" bash`.
- [ ] Expected on this machine until Android SDK is complete: failure at `ANDROID_NDK` if the NDK is not installed, or no ABI if no device is attached.

## Task 2: Definitions And Utilities

**Files:**
- Create: `rust/oboe_rust_core/src/definitions.rs`
- Create: `rust/oboe_rust_core/src/utilities.rs`
- Modify: `rust/oboe_rust_core/src/lib.rs`
- Modify: `src/common/Utilities.cpp`
- Modify: `include/oboe/AudioClock.h`

- [ ] Move format size, text conversion, compressed format checks, PCM conversion, clock reads, and sleeps into Rust.
- [ ] C++ `Utilities.cpp` and `AudioClock.h` must become thin calls into Rust.
- [ ] Run: `cargo test` in `rust/oboe_rust_core`.
- [ ] Run: single-file C++ compile checks for `src/common/Utilities.cpp`.

## Task 3: FIFO

**Files:**
- Create: `rust/oboe_rust_core/src/fifo.rs`
- Modify: `rust/oboe_rust_core/src/lib.rs`
- Modify: `src/rust/oboe_rust_core.h`
- Modify: `src/fifo/FifoControllerBase.cpp`
- Modify: `src/fifo/FifoBuffer.cpp`

- [ ] Write Rust tests for counter clipping, read/write index wraparound, write truncation on full buffers, read truncation on empty buffers, and `readNow` zero fill.
- [ ] Implement Rust FIFO operations behind C ABI helpers.
- [ ] Replace C++ FIFO math and byte-copy decisions with Rust calls while preserving C++ storage ownership.
- [ ] Run: `cargo test`.
- [ ] Run: C++ single-file compile checks for FIFO files.

## Task 4: Flowgraph

**Files:**
- Create: `rust/oboe_rust_core/src/flowgraph.rs`
- Modify: `src/flowgraph/*.cpp`
- Modify: `src/flowgraph/*.h` only when state handles need to cross the C ABI.

- [ ] Port `FlowGraphNode`, float ports, sources, sinks, channel converters, clip, limiter, mono blend, and ramp linear.
- [ ] Keep C++ constructors/destructors source-compatible and store opaque Rust handles where class state moves to Rust.
- [ ] Run: `cargo test`.
- [ ] Run original `tests/testFlowgraph.cpp` through Android test runner when NDK/device are available.

## Task 5: Resampler

**Files:**
- Create: `rust/oboe_rust_core/src/resampler.rs`
- Modify: `src/flowgraph/resampler/*.cpp`

- [ ] Port integer ratio, linear, polyphase mono/stereo, sinc mono/stereo, and multichannel resampler logic.
- [ ] Preserve current quality enum behavior and frame-loss expectations from `tests/testResampler.cpp`.
- [ ] Run Rust sweep tests mirroring each existing `test_resampler` case.
- [ ] Run original `tests/testResampler.cpp` through Android test runner when NDK/device are available.

## Task 6: Stream Core

**Files:**
- Create: `rust/oboe_rust_core/src/stream.rs`
- Create: `rust/oboe_rust_core/src/callback.rs`
- Modify: `src/common/AudioStream.cpp`
- Modify: `src/common/AudioStreamBuilder.cpp`
- Modify: `src/common/LatencyTuner.cpp`
- Modify: `src/common/StabilizedCallback.cpp`
- Modify: `src/common/OboeExtensions.cpp`

- [ ] Move builder settings, stream state transitions, close/release semantics, latency calculation, buffer size tuning policy, callback accounting, and closed-stream return values into Rust.
- [ ] C++ streams hold an opaque Rust stream handle and forward methods.
- [ ] Run stream closed-method and state tests through Android test runner when NDK/device are available.

## Task 7: Android Backends

**Files:**
- Create: `rust/oboe_rust_core/src/backend/aaudio.rs`
- Create: `rust/oboe_rust_core/src/backend/opensles.rs`
- Modify: `src/aaudio/*.cpp`
- Modify: `src/opensles/*.cpp`

- [ ] Port AAudio dynamic symbol loading, builder setup, stream read/write, callbacks, state waits, timestamp, offload, spatialization, capture policy, privacy, and extension calls.
- [ ] Port OpenSL ES engine, output mixer, buffered stream, input stream, output stream, recording/playback callbacks, and blocking I/O buffering.
- [ ] Keep only C++ ABI adapters for public class method dispatch.
- [ ] Run full `tests/run_tests.sh` on each ABI with an attached device or emulator.

## Task 8: Verification Matrix

**Files:**
- Modify: `docs/superpowers/plans/2026-05-01-full-rust-oboe-rewrite.md`

- [ ] Rust local: `cargo fmt -- --check`.
- [ ] Rust local: `cargo clippy --release --lib -- -D warnings`.
- [ ] Rust local: `cargo clippy --tests -- -D warnings`.
- [ ] Rust local: `cargo test`.
- [ ] CMake local: `cmake -S . -B /tmp/oboe-cmake-check -DOBOE_USE_RUST_CORE=ON`.
- [ ] Android: `tests/run_tests.sh` with `ANDROID_NDK`, `cmake`, `adb`, and an attached device or emulator.
