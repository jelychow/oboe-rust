# Rust-Native Oboe Android Design

## Decision

The migration target is a Rust-native Android audio library with a Kotlin/Java
wrapper. The existing C++ Oboe public API is not a long-term compatibility
requirement for this migration path.

The previous Rust-core plan preserved the C++ class hierarchy and moved behavior
behind `oboe_rust_*` C ABI calls. That approach is now superseded for the main
path because it keeps `src` as the behavioral owner. From this point, `src` is
treated as legacy reference code unless a file is explicitly part of the new
Rust/JNI build surface.

## Goals

- Replace the `src` implementation path with Rust-native implementation crates.
- Support both AAudio and OpenSL ES in the first replacement phase.
- Provide a Rust crate API for native Rust callers.
- Provide a Kotlin/Java Android wrapper as the primary Android app API.
- Move stream builder policy, stream state, callback dispatch, buffer handling,
  FIFO, flowgraph conversion, resampling, backend lifecycle, and backend I/O into
  Rust.
- Establish Rust and Android wrapper tests as the primary acceptance gates.

## Non-Goals

- Preserve source compatibility with `include/oboe/*.h`.
- Keep the existing C++ GoogleTest suite as the only or primary gate.
- Continue expanding C++ shim code as the migration strategy.
- Add third-party Rust crates in the first phase.
- Rewrite Java/Kotlin sample apps before the new wrapper API is usable.

## Architecture

The new implementation is split into four crates or modules.

1. `oboe-core`
   - Owns platform-independent audio concepts.
   - Includes stream builder settings, stream state machine, callback contracts,
     buffer ownership rules, FIFO, format conversion, channel conversion,
     flowgraph nodes, resampler implementations, latency policy, and error
     mapping.

2. `oboe-android`
   - Owns Android backend implementations.
   - Includes AAudio FFI, OpenSL ES FFI, backend selection, backend-specific
     properties, timestamp/query functions, read/write, start/stop/pause/flush,
     close/release, and callback registration.

3. `oboe-jni`
   - Owns JNI handle management and Java/Kotlin callback bridging.
   - Exposes stable JNI functions that wrap Rust stream handles.
   - Converts Java-side builder values into Rust builder settings and converts
     Rust results into Java exceptions or result codes.

4. Android wrapper module
   - Provides Kotlin/Java API classes such as `AudioStreamBuilder`,
     `AudioStream`, `AudioCallback`, and data buffer helpers.
   - Uses JNI only as a transport layer; API semantics live in Rust.

## Public API Shape

The main application-facing API is Kotlin/Java.

- `AudioStreamBuilder`
  - Direction, sharing mode, performance mode, sample rate, channel count,
    channel mask, format, input preset, usage, content type, session id,
    frames per callback, buffer capacity, package name, attribution tag,
    privacy/capture/spatialization settings, backend preference, and callback.

- `AudioStream`
  - `requestStart`, `requestPause`, `requestFlush`, `requestStop`, `close`,
    `read`, `write`, `getState`, `waitForStateChange`, `getTimestamp`,
    `getFramesRead`, `getFramesWritten`, `getBufferSizeInFrames`,
    `setBufferSizeInFrames`, `getXRunCount`, `getSampleRate`,
    `getChannelCount`, `getFormat`, and backend/property getters.

- `AudioCallback`
  - Data callback and error callback.
  - Partial data, presentation end, and routing callbacks are represented in
    Rust and surfaced through wrapper callbacks where Android API level allows.

The Rust API mirrors the same concepts but uses Rust result types and typed
builder structs. JNI is an adapter over the Rust API, not the source of truth.

## Backend Design

### AAudio

AAudio is implemented through Rust FFI bindings to Android NDK symbols. The
backend owns builder creation, stream opening, callback registration, state
requests, wait operations, read/write calls, timestamp queries, buffer tuning,
release, offload parameters, playback parameters, routing, and close/destroy.

Dynamic symbol loading remains supported where Android API levels require it,
but the loader is implemented in Rust. C++ `AAudioLoader` is not part of the
new main path.

### OpenSL ES

OpenSL ES is implemented through Rust FFI bindings to `SLES/OpenSLES.h` and
Android OpenSL ES extension interfaces. Rust owns the engine, output mixer,
player/recorder objects, buffer queues, callback registration, queue depth,
play/record state, position queries, and close/destroy order.

Buffered blocking read/write behavior is part of Rust core/backend state, not
`AudioStreamBuffered` C++ state.

## `src` Retirement Rule

The `src` directory is no longer the main implementation target.

- No new behavior should be added to C++ files under `src`.
- Existing C++ files may be read as migration reference.
- If a C++ file must remain temporarily, it must be classified as one of:
  `legacy-reference`, `build-bridge`, or `deleted-after-rust-parity`.
- New code for stream behavior, backend behavior, callbacks, buffering,
  conversion, resampling, and platform policy must be written in Rust.

The first implementation plan should include a migration matrix that assigns
each `src` subtree to a Rust module and a retirement status.

## Build Design

The new build must produce:

- Rust library artifacts for Android ABIs.
- JNI shared library loaded by the Android wrapper.
- Android wrapper package usable from Kotlin/Java tests.

The old C++ `liboboe` target is not the primary artifact. It can remain
temporarily only if needed to compare behavior during migration.

## Testing Strategy

Primary gates:

- Rust unit tests for core state, builder validation, FIFO, flowgraph,
  resampler, error mapping, callback decisions, and backend-independent policy.
- Rust fake-backend tests for AAudio and OpenSL lifecycle sequencing.
- Android device/emulator tests for real AAudio and OpenSL open/start/stop,
  callback, blocking read/write, timestamp, buffer sizing, close, and error
  behavior.
- Kotlin/Java wrapper tests for builder API, stream lifecycle, callback
  delivery, exception/result mapping, and handle cleanup.

Compatibility/reference gates:

- Existing C++ tests may be used to identify expected behavior.
- Existing C++ tests are not allowed to block the Rust-native API if their only
  failure is loss of C++ source compatibility.

## Acceptance Criteria

- A Kotlin/Java app test can open, start, stop, and close an output stream using
  AAudio.
- A Kotlin/Java app test can open, start, stop, and close an output stream using
  OpenSL ES.
- A Kotlin/Java app test can open, start, stop, and close an input stream using
  AAudio.
- A Kotlin/Java app test can open, start, stop, and close an input stream using
  OpenSL ES.
- Data callbacks are delivered through the wrapper and are owned by Rust/JNI
  callback state.
- Blocking read/write works through the wrapper for input/output streams.
- Rust tests cover builder validation, state transitions, FIFO, conversion,
  resampler, and backend fake-platform lifecycle behavior.
- `src` has a migration matrix that shows no unclassified C++ implementation
  file remains in the new main path.

## Risks

- This is a breaking migration. Downstream C++ users must stay on the old path
  or migrate to the new Rust/Kotlin API.
- AAudio and OpenSL callback lifetimes are high risk. JNI global references,
  Rust handles, and backend close order must be tested under callback shutdown.
- OpenSL ES is pointer-heavy and callback-driven. Rust wrappers must keep object
  destruction order explicit and avoid panics across FFI.
- Device coverage is required for real confidence. Local non-device tests can
  prove build and policy behavior, but not Android audio runtime behavior.

## Implementation Entry Point

The next artifact should be an implementation plan for the first replacement
phase. It must start with the `src` migration matrix, then build the new Rust
core/API skeleton, then add AAudio and OpenSL backends, then add JNI and the
Kotlin/Java wrapper tests.
