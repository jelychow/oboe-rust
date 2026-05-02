# AAudio Output Rust Vertical Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move the AAudio output stream open/start/stop/close/read-write/state/callback lifecycle behind a Rust-owned stream handle while preserving the existing C++ public API and original test suite.

**Architecture:** Rust owns a new opaque AAudio output handle and calls platform AAudio through a C ABI function table supplied by the C++ adapter. `AudioStreamAAudio` remains the source-compatible C++ class, but for output streams under `OBOE_USE_RUST_CORE=ON` it forwards lifecycle operations to Rust and keeps only callback trampolines, ADPF hooks, and public API compatibility state.

**Tech Stack:** Rust 2021 `staticlib`, no new dependencies, C ABI bridge, existing AAudioLoader dynamic symbols, existing CMake Android build, existing GoogleTest Android runner.

---

## File Structure

- `rust/oboe_rust_core/src/aaudio.rs`: add Rust-owned `AaudioOutputStream`, platform function table, output settings/properties structs, open/start/stop/close/read-write/state/wait/timestamp/buffer/xrun/frames functions, and Rust unit tests with fake platform callbacks.
- `src/rust/oboe_rust_core.h`: add ABI structs and function declarations for the Rust-owned AAudio output handle.
- `src/aaudio/AudioStreamAAudio.h`: add an opaque Rust output handle member and small private helpers guarded by `OBOE_USE_RUST_CORE`.
- `src/aaudio/AudioStreamAAudio.cpp`: add static AAudioLoader wrapper functions and forward output stream lifecycle methods to Rust when the handle is active.
- `docs/superpowers/plans/2026-05-02-aaudio-output-rust-vertical-slice.md`: this execution record.

## Task 1: Rust AAudio Output Lifecycle Tests

**Files:**
- Modify: `rust/oboe_rust_core/src/aaudio.rs`

- [ ] Add fake platform callbacks that record builder setters and simulate an opened output stream pointer.
- [ ] Add a test that `oboe_rust_aaudio_output_open` creates a Rust handle, sets output direction, writes requested format/rate/channel/buffer attributes, opens the stream, caches queried properties, and deletes the builder.
- [ ] Add a test that start/stop/close/write/state/frames calls go through the Rust handle and return `ErrorClosed` after close.
- [ ] Run: `cargo test --manifest-path rust/oboe_rust_core/Cargo.toml aaudio_output -- --nocapture`
- [ ] Expected before implementation: compile failure or unresolved symbol for the new AAudio output ABI.

## Task 2: Rust-Owned AAudio Output Handle

**Files:**
- Modify: `rust/oboe_rust_core/src/aaudio.rs`
- Modify: `src/rust/oboe_rust_core.h`

- [ ] Define `OboeRustAAudioPlatform`, `OboeRustAAudioOutputSettings`, `OboeRustAAudioOutputProperties`, and opaque `OboeRustAAudioOutputStream`.
- [ ] Implement `oboe_rust_aaudio_output_open` using the platform function table.
- [ ] Implement forwarding functions for `request_start`, `request_stop`, `request_pause`, `request_flush`, `close`, `write`, `read`, `wait_for_state_change`, `get_state`, `set_buffer_size`, `get_buffer_size`, `get_xrun_count`, `get_timestamp`, `get_frames_read`, `get_frames_written`, `get_raw_stream`, and `destroy`.
- [ ] Keep closed-stream return values compatible with existing Oboe `Result::ErrorClosed` behavior.
- [ ] Run: `cargo test --manifest-path rust/oboe_rust_core/Cargo.toml aaudio_output -- --nocapture`
- [ ] Expected after implementation: new Rust tests pass.

## Task 3: C++ AAudio Output Adapter Wiring

**Files:**
- Modify: `src/aaudio/AudioStreamAAudio.h`
- Modify: `src/aaudio/AudioStreamAAudio.cpp`

- [ ] Add `mRustAAudioOutputStream` guarded by `OBOE_USE_RUST_CORE`.
- [ ] Add C++ wrapper functions around `AAudioLoader` entries and pass them to Rust in a platform table.
- [ ] In `AudioStreamAAudio::open`, use the Rust output path when `mDirection == Direction::Output`; leave input on the existing C++ path.
- [ ] After Rust open succeeds, copy returned properties into the existing C++ fields so getters and closed-stream tests keep working.
- [ ] Forward output lifecycle methods to Rust when `mRustAAudioOutputStream != nullptr`.
- [ ] Keep C++ callback trampolines (`oboe_aaudio_data_callback_proc`, partial callback, error callback, presentation callback, routing callback) as compatibility glue.

## Task 4: Original Test Verification

**Files:**
- Existing tests under `tests/`

- [ ] Run Rust formatting: `cargo fmt --manifest-path rust/oboe_rust_core/Cargo.toml -- --check`
- [ ] Run Rust tests: `cargo test --manifest-path rust/oboe_rust_core/Cargo.toml`
- [ ] Run Rust clippy: `cargo clippy --manifest-path rust/oboe_rust_core/Cargo.toml --tests -- -D warnings`
- [ ] Run Android CMake configure/build with `OBOE_USE_RUST_CORE=ON`.
- [ ] Run original targeted Android tests from `tests/run_tests.sh` or the existing manual GTest runner when an emulator/device is available, prioritizing `StreamStates`, `StreamClosedReturnValues`, `StreamStop`, `StreamWaitState`, `ReturnStop`, and `XRunBehaviour`.

## Self-Review

- Scope is intentionally output-only for this first vertical slice; AAudio input and OpenSL remain compatibility paths until the next migration slice.
- No new runtime dependency is introduced.
- Every production ABI added in Rust has a Rust fake-platform test before C++ wiring.
- Existing C++ API remains intact because public headers and builder entry points are unchanged.
