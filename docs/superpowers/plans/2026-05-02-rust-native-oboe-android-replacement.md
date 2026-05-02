# Rust-Native Oboe Android Replacement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the `src` implementation path with a Rust-native Android audio stack that exposes Rust crate APIs and a Kotlin/Java wrapper, with AAudio and OpenSL ES both supported in the first replacement phase.

**Architecture:** Rust crates own stream policy, buffering, conversion, resampling, callback state, and Android backend lifecycles. JNI and Java are adapters over Rust handles. The existing C++ `src` tree is treated as legacy reference code and is removed from the primary build path once Rust/JNI artifacts compile and smoke tests pass.

**Tech Stack:** Rust 2021 `rlib`/`cdylib`, Android NDK FFI, raw JNI FFI without third-party Rust crates, Java Android library wrapper, Gradle Android plugin 8.5.1, CMake only for legacy/reference builds.

---

## Scope Check

The approved design intentionally covers several subsystems in one replacement phase because the user selected the one-shot replacement path. To keep execution reviewable, this plan sequences the phase into independent checkpoints. Each checkpoint must compile or test before the next one starts.

## `src` Migration Matrix

| Current `src` area | New Rust owner | First-phase retirement status | Verification owner |
| --- | --- | --- | --- |
| `src/common/AudioStream*` | `rust/oboe-core/src/stream.rs`, `rust/oboe-core/src/builder.rs` | `deleted-after-rust-parity` | Rust stream tests and Java lifecycle tests |
| `src/common/Utilities.cpp` | `rust/oboe-core/src/format.rs`, `rust/oboe-core/src/time.rs` | `deleted-after-rust-parity` | Rust format/time tests |
| `src/common/LatencyTuner.cpp` | `rust/oboe-core/src/latency.rs` | `deleted-after-rust-parity` | Rust latency policy tests |
| `src/common/StabilizedCallback.cpp` | `rust/oboe-core/src/callback.rs` | `deleted-after-rust-parity` | Rust callback tests |
| `src/fifo/*` | `rust/oboe-core/src/fifo.rs` | `deleted-after-rust-parity` | Rust FIFO tests |
| `src/flowgraph/*` | `rust/oboe-core/src/flowgraph.rs`, `rust/oboe-core/src/convert.rs` | `deleted-after-rust-parity` | Rust conversion and flowgraph tests |
| `src/flowgraph/resampler/*` | `rust/oboe-core/src/resampler.rs` | `deleted-after-rust-parity` | Rust resampler sweep tests |
| `src/aaudio/*` | `rust/oboe-android/src/aaudio.rs` | `legacy-reference` | Fake backend tests and Android AAudio smoke tests |
| `src/opensles/*` | `rust/oboe-android/src/opensles.rs` | `legacy-reference` | Fake backend tests and Android OpenSL smoke tests |
| `src/rust/oboe_rust_core.h` | `rust/oboe-jni/src/lib.rs` and Java native declarations | `deleted-after-rust-parity` | JNI symbol and Java wrapper tests |

No unclassified `src` implementation file remains in the new main path. Files under `src` may be read during porting, but they are not extended as primary implementation.

## Target File Structure

- Create: `rust/Cargo.toml` workspace for the new crates.
- Create: `rust/oboe-core/Cargo.toml`
- Create: `rust/oboe-core/src/lib.rs`
- Create: `rust/oboe-core/src/error.rs`
- Create: `rust/oboe-core/src/types.rs`
- Create: `rust/oboe-core/src/builder.rs`
- Create: `rust/oboe-core/src/stream.rs`
- Create: `rust/oboe-core/src/callback.rs`
- Create: `rust/oboe-core/src/fifo.rs`
- Create: `rust/oboe-core/src/format.rs`
- Create: `rust/oboe-core/src/resampler.rs`
- Create: `rust/oboe-android/Cargo.toml`
- Create: `rust/oboe-android/src/lib.rs`
- Create: `rust/oboe-android/src/backend.rs`
- Create: `rust/oboe-android/src/aaudio.rs`
- Create: `rust/oboe-android/src/opensles.rs`
- Create: `rust/oboe-android/src/fake.rs`
- Create: `rust/oboe-jni/Cargo.toml`
- Create: `rust/oboe-jni/src/lib.rs`
- Create: `android/oboe-wrapper/settings.gradle`
- Create: `android/oboe-wrapper/build.gradle`
- Create: `android/oboe-wrapper/oboe-wrapper/build.gradle`
- Create: `android/oboe-wrapper/oboe-wrapper/src/main/AndroidManifest.xml`
- Create: `android/oboe-wrapper/oboe-wrapper/src/main/java/com/google/oboe/AudioApi.java`
- Create: `android/oboe-wrapper/oboe-wrapper/src/main/java/com/google/oboe/AudioStreamBuilder.java`
- Create: `android/oboe-wrapper/oboe-wrapper/src/main/java/com/google/oboe/AudioStream.java`
- Create: `android/oboe-wrapper/oboe-wrapper/src/main/java/com/google/oboe/AudioCallback.java`
- Create: `android/oboe-wrapper/oboe-wrapper/src/androidTest/java/com/google/oboe/AudioStreamSmokeTest.java`
- Modify: `CMakeLists.txt` to stop treating `src` as the Rust-native main artifact.
- Modify: `rust/oboe_rust_core/Cargo.toml` only if the legacy crate must be marked as reference-only in build docs.

## Task 1: Workspace Skeleton

**Files:**
- Create: `rust/Cargo.toml`
- Create: `rust/oboe-core/Cargo.toml`
- Create: `rust/oboe-core/src/lib.rs`
- Create: `rust/oboe-android/Cargo.toml`
- Create: `rust/oboe-android/src/lib.rs`
- Create: `rust/oboe-jni/Cargo.toml`
- Create: `rust/oboe-jni/src/lib.rs`

- [ ] **Step 1: Create the Rust workspace manifest**

Write `rust/Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "oboe-core",
    "oboe-android",
    "oboe-jni",
]

[workspace.package]
edition = "2021"
license = "Apache-2.0"
version = "0.1.0"
```

- [ ] **Step 2: Create `oboe-core` manifest and empty crate root**

Write `rust/oboe-core/Cargo.toml`:

```toml
[package]
name = "oboe-core"
edition.workspace = true
license.workspace = true
version.workspace = true

[lib]
crate-type = ["rlib"]
```

Write `rust/oboe-core/src/lib.rs`:

```rust
#![deny(unsafe_op_in_unsafe_fn)]

pub const VERSION_CODE: i32 = 1;
```

- [ ] **Step 3: Create `oboe-android` manifest and crate root**

Write `rust/oboe-android/Cargo.toml`:

```toml
[package]
name = "oboe-android"
edition.workspace = true
license.workspace = true
version.workspace = true

[dependencies]
oboe-core = { path = "../oboe-core" }

[lib]
crate-type = ["rlib"]
```

Write `rust/oboe-android/src/lib.rs`:

```rust
#![deny(unsafe_op_in_unsafe_fn)]

pub fn version_code() -> i32 {
    oboe_core::VERSION_CODE
}
```

- [ ] **Step 4: Create `oboe-jni` manifest and crate root**

Write `rust/oboe-jni/Cargo.toml`:

```toml
[package]
name = "oboe-jni"
edition.workspace = true
license.workspace = true
version.workspace = true

[dependencies]
oboe-android = { path = "../oboe-android" }
oboe-core = { path = "../oboe-core" }

[lib]
crate-type = ["cdylib"]
```

Write `rust/oboe-jni/src/lib.rs`:

```rust
#![deny(unsafe_op_in_unsafe_fn)]

#[allow(non_camel_case_types)]
type jint = i32;

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeVersionCode() -> jint {
    oboe_android::version_code()
}
```

- [ ] **Step 5: Verify workspace compile failure is gone**

Run:

```bash
cargo test --manifest-path rust/Cargo.toml
```

Expected: all three crates compile; no tests run yet.

- [ ] **Step 6: Commit the skeleton**

```bash
git add rust/Cargo.toml rust/oboe-core rust/oboe-android rust/oboe-jni
git commit -m "Establish Rust-native Oboe workspace" \
  -m "The replacement path needs independent Rust crates before any src retirement can be reviewed." \
  -m "Constraint: No third-party Rust crates in the first replacement phase" \
  -m "Confidence: high" \
  -m "Scope-risk: moderate" \
  -m "Tested: cargo test --manifest-path rust/Cargo.toml" \
  -m "Not-tested: Android JNI loading"
```

## Task 2: Core Types, Errors, And Builder

**Files:**
- Create: `rust/oboe-core/src/error.rs`
- Create: `rust/oboe-core/src/types.rs`
- Create: `rust/oboe-core/src/builder.rs`
- Modify: `rust/oboe-core/src/lib.rs`

- [ ] **Step 1: Write failing builder tests**

Append to `rust/oboe-core/src/builder.rs` while creating the file:

```rust
use crate::error::{Error, Result};
use crate::types::{AudioApi, Direction, Format, PerformanceMode, SharingMode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamBuilder {
    pub api: AudioApi,
    pub direction: Direction,
    pub sharing_mode: SharingMode,
    pub performance_mode: PerformanceMode,
    pub sample_rate: i32,
    pub channel_count: i32,
    pub format: Format,
    pub frames_per_callback: i32,
    pub buffer_capacity_in_frames: i32,
}

impl Default for StreamBuilder {
    fn default() -> Self {
        Self {
            api: AudioApi::Unspecified,
            direction: Direction::Output,
            sharing_mode: SharingMode::Shared,
            performance_mode: PerformanceMode::None,
            sample_rate: 0,
            channel_count: 2,
            format: Format::Float,
            frames_per_callback: 0,
            buffer_capacity_in_frames: 0,
        }
    }
}

impl StreamBuilder {
    pub fn validate(&self) -> Result<()> {
        Err(Error::Unimplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_builder_is_valid_output_float_stream() {
        let builder = StreamBuilder::default();
        assert_eq!(builder.validate(), Ok(()));
    }

    #[test]
    fn rejects_negative_sample_rate() {
        let builder = StreamBuilder {
            sample_rate: -48_000,
            ..StreamBuilder::default()
        };
        assert_eq!(builder.validate(), Err(Error::InvalidArgument));
    }

    #[test]
    fn rejects_zero_channel_count() {
        let builder = StreamBuilder {
            channel_count: 0,
            ..StreamBuilder::default()
        };
        assert_eq!(builder.validate(), Err(Error::InvalidArgument));
    }
}
```

- [ ] **Step 2: Add supporting enums and errors**

Write `rust/oboe-core/src/error.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidArgument,
    InvalidState,
    Closed,
    Unavailable,
    BackendUnavailable,
    Internal,
    Unimplemented,
}

pub type Result<T> = core::result::Result<T, Error>;
```

Write `rust/oboe-core/src/types.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioApi {
    Unspecified,
    AAudio,
    OpenSLES,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Input,
    Output,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SharingMode {
    Shared,
    Exclusive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PerformanceMode {
    None,
    PowerSaving,
    LowLatency,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Unspecified,
    I16,
    I24,
    I32,
    Float,
}
```

- [ ] **Step 3: Update the core crate root**

Replace `rust/oboe-core/src/lib.rs`:

```rust
#![deny(unsafe_op_in_unsafe_fn)]

pub mod builder;
pub mod error;
pub mod types;

pub const VERSION_CODE: i32 = 1;
```

- [ ] **Step 4: Run tests and verify the planned failure**

Run:

```bash
cargo test --manifest-path rust/Cargo.toml -p oboe-core builder
```

Expected: `default_builder_is_valid_output_float_stream` fails because `validate()` returns `Unimplemented`.

- [ ] **Step 5: Implement validation**

Replace `StreamBuilder::validate` in `rust/oboe-core/src/builder.rs`:

```rust
impl StreamBuilder {
    pub fn validate(&self) -> Result<()> {
        if self.sample_rate < 0 {
            return Err(Error::InvalidArgument);
        }
        if self.channel_count <= 0 {
            return Err(Error::InvalidArgument);
        }
        if self.frames_per_callback < 0 {
            return Err(Error::InvalidArgument);
        }
        if self.buffer_capacity_in_frames < 0 {
            return Err(Error::InvalidArgument);
        }
        Ok(())
    }
}
```

- [ ] **Step 6: Verify builder tests pass**

Run:

```bash
cargo test --manifest-path rust/Cargo.toml -p oboe-core builder
```

Expected: 3 tests pass.

- [ ] **Step 7: Commit**

```bash
git add rust/oboe-core/src/error.rs rust/oboe-core/src/types.rs rust/oboe-core/src/builder.rs rust/oboe-core/src/lib.rs
git commit -m "Define Rust-native stream builder contract" \
  -m "The wrapper and Android backends need one typed builder before backend selection is moved out of src." \
  -m "Constraint: The new API does not preserve include/oboe source compatibility" \
  -m "Confidence: high" \
  -m "Scope-risk: narrow" \
  -m "Tested: cargo test --manifest-path rust/Cargo.toml -p oboe-core builder" \
  -m "Not-tested: Backend open behavior"
```

## Task 3: Stream State And Backend Trait

**Files:**
- Create: `rust/oboe-core/src/stream.rs`
- Create: `rust/oboe-android/src/backend.rs`
- Create: `rust/oboe-android/src/fake.rs`
- Modify: `rust/oboe-android/src/lib.rs`

- [ ] **Step 1: Write stream state tests**

Write `rust/oboe-core/src/stream.rs`:

```rust
use crate::error::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamState {
    Uninitialized,
    Open,
    Starting,
    Started,
    Pausing,
    Paused,
    Flushing,
    Flushed,
    Stopping,
    Stopped,
    Closed,
}

#[derive(Debug)]
pub struct StreamCore {
    state: StreamState,
}

impl StreamCore {
    pub fn new_open() -> Self {
        Self {
            state: StreamState::Open,
        }
    }

    pub fn state(&self) -> StreamState {
        self.state
    }

    pub fn request_start(&mut self) -> Result<()> {
        Err(Error::Unimplemented)
    }

    pub fn request_stop(&mut self) -> Result<()> {
        Err(Error::Unimplemented)
    }

    pub fn close(&mut self) -> Result<()> {
        Err(Error::Unimplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_start_stop_close_sequence_is_owned_by_core() {
        let mut stream = StreamCore::new_open();
        assert_eq!(stream.request_start(), Ok(()));
        assert_eq!(stream.state(), StreamState::Started);
        assert_eq!(stream.request_stop(), Ok(()));
        assert_eq!(stream.state(), StreamState::Stopped);
        assert_eq!(stream.close(), Ok(()));
        assert_eq!(stream.state(), StreamState::Closed);
    }

    #[test]
    fn closed_stream_rejects_start() {
        let mut stream = StreamCore::new_open();
        assert_eq!(stream.close(), Ok(()));
        assert_eq!(stream.request_start(), Err(Error::Closed));
    }
}
```

- [ ] **Step 2: Update the core crate root for stream tests**

Replace `rust/oboe-core/src/lib.rs`:

```rust
#![deny(unsafe_op_in_unsafe_fn)]

pub mod builder;
pub mod error;
pub mod stream;
pub mod types;

pub const VERSION_CODE: i32 = 1;
```

- [ ] **Step 3: Run state tests and verify the planned failure**

Run:

```bash
cargo test --manifest-path rust/Cargo.toml -p oboe-core stream
```

Expected: both tests fail because stream methods return `Unimplemented`.

- [ ] **Step 4: Implement state transitions**

Replace the three method bodies in `rust/oboe-core/src/stream.rs`:

```rust
    pub fn request_start(&mut self) -> Result<()> {
        match self.state {
            StreamState::Closed => Err(Error::Closed),
            StreamState::Started => Ok(()),
            _ => {
                self.state = StreamState::Started;
                Ok(())
            }
        }
    }

    pub fn request_stop(&mut self) -> Result<()> {
        match self.state {
            StreamState::Closed => Err(Error::Closed),
            StreamState::Stopped => Ok(()),
            _ => {
                self.state = StreamState::Stopped;
                Ok(())
            }
        }
    }

    pub fn close(&mut self) -> Result<()> {
        self.state = StreamState::Closed;
        Ok(())
    }
```

- [ ] **Step 5: Add backend trait and fake backend tests**

Write `rust/oboe-android/src/backend.rs`:

```rust
use oboe_core::builder::StreamBuilder;
use oboe_core::error::Result;
use oboe_core::stream::StreamState;

pub trait AudioBackend {
    fn open(builder: &StreamBuilder) -> Result<Self>
    where
        Self: Sized;
    fn request_start(&mut self) -> Result<()>;
    fn request_stop(&mut self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
    fn state(&self) -> StreamState;
}
```

Write `rust/oboe-android/src/fake.rs`:

```rust
use crate::backend::AudioBackend;
use oboe_core::builder::StreamBuilder;
use oboe_core::error::Result;
use oboe_core::stream::{StreamCore, StreamState};

#[derive(Debug)]
pub struct FakeBackend {
    core: StreamCore,
}

impl AudioBackend for FakeBackend {
    fn open(builder: &StreamBuilder) -> Result<Self> {
        builder.validate()?;
        Ok(Self {
            core: StreamCore::new_open(),
        })
    }

    fn request_start(&mut self) -> Result<()> {
        self.core.request_start()
    }

    fn request_stop(&mut self) -> Result<()> {
        self.core.request_stop()
    }

    fn close(&mut self) -> Result<()> {
        self.core.close()
    }

    fn state(&self) -> StreamState {
        self.core.state()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_backend_proves_backend_trait_contract() {
        let mut backend = FakeBackend::open(&StreamBuilder::default()).unwrap();
        assert_eq!(backend.state(), StreamState::Open);
        assert_eq!(backend.request_start(), Ok(()));
        assert_eq!(backend.state(), StreamState::Started);
        assert_eq!(backend.request_stop(), Ok(()));
        assert_eq!(backend.state(), StreamState::Stopped);
        assert_eq!(backend.close(), Ok(()));
        assert_eq!(backend.state(), StreamState::Closed);
    }
}
```

- [ ] **Step 6: Update the Android crate root for backend modules**

Replace `rust/oboe-android/src/lib.rs`:

```rust
#![deny(unsafe_op_in_unsafe_fn)]

pub mod backend;
pub mod fake;

pub fn version_code() -> i32 {
    oboe_core::VERSION_CODE
}
```

- [ ] **Step 7: Verify core and fake backend tests pass**

Run:

```bash
cargo test --manifest-path rust/Cargo.toml -p oboe-core stream
cargo test --manifest-path rust/Cargo.toml -p oboe-android fake_backend
```

Expected: all targeted tests pass.

- [ ] **Step 8: Commit**

```bash
git add rust/oboe-core/src/lib.rs rust/oboe-core/src/stream.rs rust/oboe-android/src/backend.rs rust/oboe-android/src/fake.rs rust/oboe-android/src/lib.rs
git commit -m "Move stream state contract into Rust" \
  -m "AAudio, OpenSL, JNI, and Java need one backend-neutral lifecycle contract before platform calls are added." \
  -m "Constraint: Closed-stream behavior must be decided in Rust, not src/common" \
  -m "Confidence: high" \
  -m "Scope-risk: moderate" \
  -m "Tested: cargo test --manifest-path rust/Cargo.toml -p oboe-core stream; cargo test --manifest-path rust/Cargo.toml -p oboe-android fake_backend" \
  -m "Not-tested: Android platform backends"
```

## Task 4: Core FIFO, Format, And Resampler Foundation

**Files:**
- Create: `rust/oboe-core/src/fifo.rs`
- Create: `rust/oboe-core/src/format.rs`
- Create: `rust/oboe-core/src/resampler.rs`
- Modify: `rust/oboe-core/src/lib.rs`

- [ ] **Step 1: Write FIFO tests and implementation**

Write `rust/oboe-core/src/fifo.rs`:

```rust
use crate::error::{Error, Result};

#[derive(Debug)]
pub struct Fifo {
    data: Vec<f32>,
    read: usize,
    write: usize,
    len: usize,
}

impl Fifo {
    pub fn with_capacity(frames: usize) -> Result<Self> {
        if frames == 0 {
            return Err(Error::InvalidArgument);
        }
        Ok(Self {
            data: vec![0.0; frames],
            read: 0,
            write: 0,
            len: 0,
        })
    }

    pub fn available_to_read(&self) -> usize {
        self.len
    }

    pub fn available_to_write(&self) -> usize {
        self.data.len() - self.len
    }

    pub fn write(&mut self, input: &[f32]) -> usize {
        let count = input.len().min(self.available_to_write());
        for sample in input.iter().take(count) {
            self.data[self.write] = *sample;
            self.write = (self.write + 1) % self.data.len();
        }
        self.len += count;
        count
    }

    pub fn read(&mut self, output: &mut [f32]) -> usize {
        let count = output.len().min(self.available_to_read());
        for slot in output.iter_mut().take(count) {
            *slot = self.data[self.read];
            self.read = (self.read + 1) % self.data.len();
        }
        self.len -= count;
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifo_truncates_writes_and_preserves_order_across_wrap() {
        let mut fifo = Fifo::with_capacity(3).unwrap();
        assert_eq!(fifo.write(&[1.0, 2.0, 3.0, 4.0]), 3);
        let mut first = [0.0; 2];
        assert_eq!(fifo.read(&mut first), 2);
        assert_eq!(first, [1.0, 2.0]);
        assert_eq!(fifo.write(&[4.0, 5.0]), 2);
        let mut second = [0.0; 3];
        assert_eq!(fifo.read(&mut second), 3);
        assert_eq!(second, [3.0, 4.0, 5.0]);
    }
}
```

- [ ] **Step 2: Write format conversion tests and implementation**

Write `rust/oboe-core/src/format.rs`:

```rust
use crate::types::Format;

pub fn bytes_per_sample(format: Format) -> usize {
    match format {
        Format::Unspecified => 0,
        Format::I16 => 2,
        Format::I24 => 3,
        Format::I32 | Format::Float => 4,
    }
}

pub fn i16_to_float(sample: i16) -> f32 {
    if sample == i16::MIN {
        -1.0
    } else {
        sample as f32 / i16::MAX as f32
    }
}

pub fn float_to_i16(sample: f32) -> i16 {
    let clipped = sample.clamp(-1.0, 1.0);
    if clipped <= -1.0 {
        i16::MIN
    } else {
        (clipped * i16::MAX as f32).round() as i16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_sample_sizes() {
        assert_eq!(bytes_per_sample(Format::I16), 2);
        assert_eq!(bytes_per_sample(Format::I24), 3);
        assert_eq!(bytes_per_sample(Format::I32), 4);
        assert_eq!(bytes_per_sample(Format::Float), 4);
    }

    #[test]
    fn converts_i16_and_float_with_clipping() {
        assert_eq!(float_to_i16(2.0), i16::MAX);
        assert_eq!(float_to_i16(-2.0), i16::MIN);
        assert!((i16_to_float(i16::MAX) - 1.0).abs() < 0.0001);
    }
}
```

- [ ] **Step 3: Write resampler foundation tests and implementation**

Write `rust/oboe-core/src/resampler.rs`:

```rust
use crate::error::{Error, Result};

pub fn linear_interpolate(previous: f32, current: f32, fraction: f32) -> Result<f32> {
    if !(0.0..=1.0).contains(&fraction) {
        return Err(Error::InvalidArgument);
    }
    Ok(previous + (current - previous) * fraction)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_midpoint() {
        assert_eq!(linear_interpolate(2.0, 6.0, 0.5), Ok(4.0));
    }

    #[test]
    fn rejects_fraction_outside_unit_interval() {
        assert_eq!(linear_interpolate(2.0, 6.0, -0.1), Err(Error::InvalidArgument));
        assert_eq!(linear_interpolate(2.0, 6.0, 1.1), Err(Error::InvalidArgument));
    }
}
```

- [ ] **Step 4: Update the core crate root for utility modules**

Replace `rust/oboe-core/src/lib.rs`:

```rust
#![deny(unsafe_op_in_unsafe_fn)]

pub mod builder;
pub mod error;
pub mod fifo;
pub mod format;
pub mod resampler;
pub mod stream;
pub mod types;

pub const VERSION_CODE: i32 = 1;
```

- [ ] **Step 5: Verify core tests**

Run:

```bash
cargo test --manifest-path rust/Cargo.toml -p oboe-core
```

Expected: builder, stream, FIFO, format, and resampler tests pass.

- [ ] **Step 6: Commit**

```bash
git add rust/oboe-core/src/fifo.rs rust/oboe-core/src/format.rs rust/oboe-core/src/resampler.rs rust/oboe-core/src/lib.rs
git commit -m "Port first core audio utilities to Rust-native API" \
  -m "FIFO, format conversion, and resampler foundations remove src/fifo and flowgraph helpers from the new main path." \
  -m "Constraint: First phase uses focused foundations before matching every legacy C++ edge case" \
  -m "Confidence: medium" \
  -m "Scope-risk: moderate" \
  -m "Tested: cargo test --manifest-path rust/Cargo.toml -p oboe-core" \
  -m "Not-tested: Full legacy resampler parity sweep"
```

## Task 5: Backend Selection With Fake AAudio And OpenSL

**Files:**
- Create: `rust/oboe-android/src/aaudio.rs`
- Create: `rust/oboe-android/src/opensles.rs`
- Modify: `rust/oboe-android/src/lib.rs`

- [ ] **Step 1: Add AAudio backend skeleton**

Write `rust/oboe-android/src/aaudio.rs`:

```rust
use crate::backend::AudioBackend;
use oboe_core::builder::StreamBuilder;
use oboe_core::error::Result;
use oboe_core::stream::{StreamCore, StreamState};

#[derive(Debug)]
pub struct AAudioBackend {
    core: StreamCore,
}

impl AudioBackend for AAudioBackend {
    fn open(builder: &StreamBuilder) -> Result<Self> {
        builder.validate()?;
        Ok(Self {
            core: StreamCore::new_open(),
        })
    }

    fn request_start(&mut self) -> Result<()> {
        self.core.request_start()
    }

    fn request_stop(&mut self) -> Result<()> {
        self.core.request_stop()
    }

    fn close(&mut self) -> Result<()> {
        self.core.close()
    }

    fn state(&self) -> StreamState {
        self.core.state()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aaudio_backend_supports_core_lifecycle_before_real_ffi() {
        let mut backend = AAudioBackend::open(&StreamBuilder::default()).unwrap();
        assert_eq!(backend.request_start(), Ok(()));
        assert_eq!(backend.state(), StreamState::Started);
        assert_eq!(backend.request_stop(), Ok(()));
        assert_eq!(backend.close(), Ok(()));
    }
}
```

- [ ] **Step 2: Add OpenSL backend skeleton**

Write `rust/oboe-android/src/opensles.rs`:

```rust
use crate::backend::AudioBackend;
use oboe_core::builder::StreamBuilder;
use oboe_core::error::Result;
use oboe_core::stream::{StreamCore, StreamState};

#[derive(Debug)]
pub struct OpenSlBackend {
    core: StreamCore,
}

impl AudioBackend for OpenSlBackend {
    fn open(builder: &StreamBuilder) -> Result<Self> {
        builder.validate()?;
        Ok(Self {
            core: StreamCore::new_open(),
        })
    }

    fn request_start(&mut self) -> Result<()> {
        self.core.request_start()
    }

    fn request_stop(&mut self) -> Result<()> {
        self.core.request_stop()
    }

    fn close(&mut self) -> Result<()> {
        self.core.close()
    }

    fn state(&self) -> StreamState {
        self.core.state()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opensl_backend_supports_core_lifecycle_before_real_ffi() {
        let mut backend = OpenSlBackend::open(&StreamBuilder::default()).unwrap();
        assert_eq!(backend.request_start(), Ok(()));
        assert_eq!(backend.state(), StreamState::Started);
        assert_eq!(backend.request_stop(), Ok(()));
        assert_eq!(backend.close(), Ok(()));
    }
}
```

- [ ] **Step 3: Update the Android crate root for both backend owners**

Replace `rust/oboe-android/src/lib.rs`:

```rust
#![deny(unsafe_op_in_unsafe_fn)]

pub mod aaudio;
pub mod backend;
pub mod fake;
pub mod opensles;

pub fn version_code() -> i32 {
    oboe_core::VERSION_CODE
}
```

- [ ] **Step 4: Verify both backend skeletons**

Run:

```bash
cargo test --manifest-path rust/Cargo.toml -p oboe-android aaudio_backend
cargo test --manifest-path rust/Cargo.toml -p oboe-android opensl_backend
```

Expected: both targeted test groups pass.

- [ ] **Step 5: Commit**

```bash
git add rust/oboe-android/src/aaudio.rs rust/oboe-android/src/opensles.rs rust/oboe-android/src/lib.rs
git commit -m "Create Rust backend owners for AAudio and OpenSL" \
  -m "Both Android audio APIs must exist in the first replacement phase before JNI exposes stream creation." \
  -m "Constraint: The skeleton intentionally avoids C++ src wrappers" \
  -m "Confidence: medium" \
  -m "Scope-risk: moderate" \
  -m "Tested: cargo test --manifest-path rust/Cargo.toml -p oboe-android aaudio_backend; cargo test --manifest-path rust/Cargo.toml -p oboe-android opensl_backend" \
  -m "Not-tested: Real Android NDK FFI"
```

## Task 6: JNI Handle API

**Files:**
- Modify: `rust/oboe-jni/src/lib.rs`
- Modify: `rust/oboe-android/src/lib.rs`

- [ ] **Step 1: Replace version-only JNI with handle functions**

Write `rust/oboe-jni/src/lib.rs`:

```rust
#![deny(unsafe_op_in_unsafe_fn)]

use oboe_android::aaudio::AAudioBackend;
use oboe_android::backend::AudioBackend;
use oboe_android::opensles::OpenSlBackend;
use oboe_core::builder::StreamBuilder;
use oboe_core::stream::StreamState;
use oboe_core::types::AudioApi;

#[allow(non_camel_case_types)]
type jint = i32;
#[allow(non_camel_case_types)]
type jlong = i64;
#[allow(non_camel_case_types)]
type jobject = *mut core::ffi::c_void;
#[allow(non_camel_case_types)]
type jclass = *mut core::ffi::c_void;
#[allow(non_camel_case_types)]
type JNIEnv = *mut core::ffi::c_void;

enum NativeStream {
    AAudio(AAudioBackend),
    OpenSl(OpenSlBackend),
}

impl NativeStream {
    fn request_start(&mut self) -> i32 {
        match self {
            NativeStream::AAudio(stream) => stream.request_start(),
            NativeStream::OpenSl(stream) => stream.request_start(),
        }
        .map(|_| 0)
        .unwrap_or(-1)
    }

    fn request_stop(&mut self) -> i32 {
        match self {
            NativeStream::AAudio(stream) => stream.request_stop(),
            NativeStream::OpenSl(stream) => stream.request_stop(),
        }
        .map(|_| 0)
        .unwrap_or(-1)
    }

    fn close(&mut self) -> i32 {
        match self {
            NativeStream::AAudio(stream) => stream.close(),
            NativeStream::OpenSl(stream) => stream.close(),
        }
        .map(|_| 0)
        .unwrap_or(-1)
    }

    fn state(&self) -> StreamState {
        match self {
            NativeStream::AAudio(stream) => stream.state(),
            NativeStream::OpenSl(stream) => stream.state(),
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeVersionCode() -> jint {
    1
}

#[no_mangle]
pub extern "system" fn Java_com_google_oboe_AudioStream_nativeOpen(
    _env: JNIEnv,
    _class: jclass,
    api: jint,
) -> jlong {
    let mut builder = StreamBuilder::default();
    builder.api = match api {
        1 => AudioApi::AAudio,
        2 => AudioApi::OpenSLES,
        _ => AudioApi::Unspecified,
    };
    let stream = match builder.api {
        AudioApi::AAudio | AudioApi::Unspecified => {
            AAudioBackend::open(&builder).map(NativeStream::AAudio)
        }
        AudioApi::OpenSLES => OpenSlBackend::open(&builder).map(NativeStream::OpenSl),
    };
    match stream {
        Ok(stream) => Box::into_raw(Box::new(stream)) as jlong,
        Err(_) => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_google_oboe_AudioStream_nativeRequestStart(
    _env: JNIEnv,
    _self: jobject,
    handle: jlong,
) -> jint {
    let Some(stream) = stream_from_handle(handle) else {
        return -1;
    };
    stream.request_start()
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_google_oboe_AudioStream_nativeRequestStop(
    _env: JNIEnv,
    _self: jobject,
    handle: jlong,
) -> jint {
    let Some(stream) = stream_from_handle(handle) else {
        return -1;
    };
    stream.request_stop()
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_google_oboe_AudioStream_nativeGetState(
    _env: JNIEnv,
    _self: jobject,
    handle: jlong,
) -> jint {
    let Some(stream) = stream_from_handle(handle) else {
        return -1;
    };
    stream.state() as jint
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_google_oboe_AudioStream_nativeClose(
    _env: JNIEnv,
    _self: jobject,
    handle: jlong,
) -> jint {
    if handle == 0 {
        return -1;
    }
    let mut stream = unsafe { Box::from_raw(handle as *mut NativeStream) };
    stream.close()
}

unsafe fn stream_from_handle<'a>(handle: jlong) -> Option<&'a mut NativeStream> {
    if handle == 0 {
        return None;
    }
    unsafe { (handle as *mut NativeStream).as_mut() }
}
```

- [ ] **Step 2: Run JNI crate tests and compile checks**

Run:

```bash
cargo test --manifest-path rust/Cargo.toml -p oboe-jni
cargo build --manifest-path rust/Cargo.toml -p oboe-jni --release
```

Expected: `oboe-jni` compiles as a host `cdylib`. Android cross build is covered after Gradle wiring.

- [ ] **Step 3: Commit**

```bash
git add rust/oboe-jni/src/lib.rs
git commit -m "Expose Rust stream handles through JNI" \
  -m "The Java wrapper needs stable native entry points that allocate, drive, and close Rust-owned AAudio/OpenSL streams." \
  -m "Constraint: JNI uses raw FFI types to avoid adding Rust dependencies" \
  -m "Confidence: medium" \
  -m "Scope-risk: moderate" \
  -m "Tested: cargo test --manifest-path rust/Cargo.toml -p oboe-jni; cargo build --manifest-path rust/Cargo.toml -p oboe-jni --release" \
  -m "Not-tested: Android runtime library loading"
```

## Task 7: Java Android Wrapper

**Files:**
- Create: `android/oboe-wrapper/settings.gradle`
- Create: `android/oboe-wrapper/build.gradle`
- Create: `android/oboe-wrapper/oboe-wrapper/build.gradle`
- Create: `android/oboe-wrapper/oboe-wrapper/src/main/AndroidManifest.xml`
- Create: `android/oboe-wrapper/oboe-wrapper/src/main/java/com/google/oboe/AudioApi.java`
- Create: `android/oboe-wrapper/oboe-wrapper/src/main/java/com/google/oboe/AudioStreamBuilder.java`
- Create: `android/oboe-wrapper/oboe-wrapper/src/main/java/com/google/oboe/AudioStream.java`
- Create: `android/oboe-wrapper/oboe-wrapper/src/main/java/com/google/oboe/AudioCallback.java`

- [ ] **Step 1: Create Gradle wrapper project files**

Write `android/oboe-wrapper/settings.gradle`:

```groovy
pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = 'RustNativeOboeWrapper'
include ':oboe-wrapper'
```

Write `android/oboe-wrapper/build.gradle`:

```groovy
plugins {
    id 'com.android.library' version '8.5.1' apply false
}
```

Write `android/oboe-wrapper/oboe-wrapper/build.gradle`:

```groovy
plugins {
    id 'com.android.library'
}

android {
    namespace 'com.google.oboe'
    compileSdk 34

    defaultConfig {
        minSdk 21
        testInstrumentationRunner 'android.test.InstrumentationTestRunner'
    }
}
```

Write `android/oboe-wrapper/oboe-wrapper/src/main/AndroidManifest.xml`:

```xml
<manifest xmlns:android="http://schemas.android.com/apk/res/android" />
```

- [ ] **Step 2: Create Java API classes**

Write `android/oboe-wrapper/oboe-wrapper/src/main/java/com/google/oboe/AudioApi.java`:

```java
package com.google.oboe;

public enum AudioApi {
    UNSPECIFIED(0),
    AAUDIO(1),
    OPENSL_ES(2);

    final int nativeValue;

    AudioApi(int nativeValue) {
        this.nativeValue = nativeValue;
    }
}
```

Write `android/oboe-wrapper/oboe-wrapper/src/main/java/com/google/oboe/AudioCallback.java`:

```java
package com.google.oboe;

public interface AudioCallback {
    int onAudioReady(AudioStream stream, float[] audioData, int numFrames);
    void onError(AudioStream stream, int error);
}
```

Write `android/oboe-wrapper/oboe-wrapper/src/main/java/com/google/oboe/AudioStreamBuilder.java`:

```java
package com.google.oboe;

public final class AudioStreamBuilder {
    private AudioApi audioApi = AudioApi.UNSPECIFIED;

    public AudioStreamBuilder setAudioApi(AudioApi audioApi) {
        if (audioApi == null) {
            throw new IllegalArgumentException("audioApi must not be null");
        }
        this.audioApi = audioApi;
        return this;
    }

    public AudioStream openStream() {
        long handle = AudioStream.nativeOpen(audioApi.nativeValue);
        if (handle == 0) {
            throw new IllegalStateException("native stream open failed");
        }
        return new AudioStream(handle);
    }
}
```

Write `android/oboe-wrapper/oboe-wrapper/src/main/java/com/google/oboe/AudioStream.java`:

```java
package com.google.oboe;

public final class AudioStream implements AutoCloseable {
    static {
        System.loadLibrary("oboe_jni");
    }

    private long nativeHandle;

    AudioStream(long nativeHandle) {
        this.nativeHandle = nativeHandle;
    }

    public static native int nativeVersionCode();
    static native long nativeOpen(int audioApi);
    private static native int nativeRequestStart(long handle);
    private static native int nativeRequestStop(long handle);
    private static native int nativeGetState(long handle);
    private static native int nativeClose(long handle);

    public int requestStart() {
        ensureOpen();
        return nativeRequestStart(nativeHandle);
    }

    public int requestStop() {
        ensureOpen();
        return nativeRequestStop(nativeHandle);
    }

    public int getState() {
        ensureOpen();
        return nativeGetState(nativeHandle);
    }

    @Override
    public void close() {
        if (nativeHandle != 0) {
            nativeClose(nativeHandle);
            nativeHandle = 0;
        }
    }

    private void ensureOpen() {
        if (nativeHandle == 0) {
            throw new IllegalStateException("stream is closed");
        }
    }
}
```

- [ ] **Step 3: Run Java compile check**

Run:

```bash
cd android/oboe-wrapper
./gradlew :oboe-wrapper:compileDebugJavaWithJavac
```

Expected: Java sources compile. If `gradlew` is missing in this new project, copy the wrapper scripts from `tests/UnitTestRunner/gradlew` and `tests/UnitTestRunner/gradle/` as part of this step, then rerun the command.

- [ ] **Step 4: Commit**

```bash
git add android/oboe-wrapper
git commit -m "Add Java wrapper for Rust-native Oboe streams" \
  -m "The new Android API is Java/Kotlin-facing and calls JNI stream handles instead of C++ Oboe classes." \
  -m "Constraint: Wrapper starts with Java to avoid adding Kotlin Gradle setup before JNI is validated" \
  -m "Confidence: medium" \
  -m "Scope-risk: moderate" \
  -m "Tested: ./gradlew :oboe-wrapper:compileDebugJavaWithJavac from android/oboe-wrapper" \
  -m "Not-tested: Native library packaging"
```

## Task 8: Android Build Packaging For `oboe_jni`

**Files:**
- Modify: `android/oboe-wrapper/oboe-wrapper/build.gradle`
- Create: `android/oboe-wrapper/oboe-wrapper/src/main/jniLibs/.gitkeep`
- Create: `tools/build-rust-android.ps1`

- [ ] **Step 1: Add a PowerShell Rust Android build script**

Write `tools/build-rust-android.ps1`:

```powershell
param(
    [Parameter(Mandatory=$true)][string]$AndroidNdk,
    [string]$ApiLevel = "21"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$RustManifest = Join-Path $Root "rust/Cargo.toml"
$OutRoot = Join-Path $Root "android/oboe-wrapper/oboe-wrapper/src/main/jniLibs"

$Targets = @(
    @{ Abi = "arm64-v8a"; Triple = "aarch64-linux-android"; Linker = "aarch64-linux-android$ApiLevel-clang.cmd" },
    @{ Abi = "armeabi-v7a"; Triple = "armv7-linux-androideabi"; Linker = "armv7a-linux-androideabi$ApiLevel-clang.cmd" },
    @{ Abi = "x86"; Triple = "i686-linux-android"; Linker = "i686-linux-android$ApiLevel-clang.cmd" },
    @{ Abi = "x86_64"; Triple = "x86_64-linux-android"; Linker = "x86_64-linux-android$ApiLevel-clang.cmd" }
)

foreach ($Target in $Targets) {
    $LinkerPath = Join-Path $AndroidNdk "toolchains/llvm/prebuilt/windows-x86_64/bin/$($Target.Linker)"
    if (!(Test-Path $LinkerPath)) {
        throw "Missing Android linker: $LinkerPath"
    }

    $EnvName = "CARGO_TARGET_" + ($Target.Triple.ToUpperInvariant() -replace "-", "_") + "_LINKER"
    Set-Item -Path "Env:$EnvName" -Value $LinkerPath
    cargo build --manifest-path $RustManifest -p oboe-jni --release --target $Target.Triple

    $Dest = Join-Path $OutRoot $Target.Abi
    New-Item -ItemType Directory -Force -Path $Dest | Out-Null
    Copy-Item -Force (Join-Path $Root "rust/target/$($Target.Triple)/release/liboboe_jni.so") (Join-Path $Dest "liboboe_jni.so")
}
```

- [ ] **Step 2: Create jniLibs marker**

Write `android/oboe-wrapper/oboe-wrapper/src/main/jniLibs/.gitkeep` as an empty file.

- [ ] **Step 3: Build JNI libraries for Android**

Run from PowerShell:

```powershell
.\tools\build-rust-android.ps1 -AndroidNdk $Env:ANDROID_NDK_HOME -ApiLevel 21
```

Expected: `liboboe_jni.so` appears under each ABI directory in `android/oboe-wrapper/oboe-wrapper/src/main/jniLibs/`.

- [ ] **Step 4: Build the wrapper APK/AAR**

Run:

```bash
cd android/oboe-wrapper
./gradlew :oboe-wrapper:assembleDebug
```

Expected: `BUILD SUCCESSFUL`.

- [ ] **Step 5: Commit**

```bash
git add tools/build-rust-android.ps1 android/oboe-wrapper/oboe-wrapper/build.gradle android/oboe-wrapper/oboe-wrapper/src/main/jniLibs
git commit -m "Package Rust JNI library for Android wrapper" \
  -m "The Java wrapper needs ABI-specific liboboe_jni.so artifacts produced from the Rust-native crate graph." \
  -m "Constraint: First packaging script targets the current Windows/PowerShell workflow" \
  -m "Confidence: medium" \
  -m "Scope-risk: moderate" \
  -m "Tested: ./tools/build-rust-android.ps1 with ANDROID_NDK_HOME; ./gradlew :oboe-wrapper:assembleDebug" \
  -m "Not-tested: Linux/macOS packaging script"
```

## Task 9: Wrapper Smoke Tests For AAudio And OpenSL

**Files:**
- Create: `android/oboe-wrapper/oboe-wrapper/src/androidTest/java/com/google/oboe/AudioStreamSmokeTest.java`

- [ ] **Step 1: Add instrumentation smoke tests**

Write `android/oboe-wrapper/oboe-wrapper/src/androidTest/java/com/google/oboe/AudioStreamSmokeTest.java`:

```java
package com.google.oboe;

import android.test.InstrumentationTestCase;

public final class AudioStreamSmokeTest extends InstrumentationTestCase {
    public void testNativeLibraryLoads() {
        assertEquals(1, AudioStream.nativeVersionCode());
    }

    public void testAAudioOutputLifecycle() {
        AudioStream stream = new AudioStreamBuilder()
                .setAudioApi(AudioApi.AAUDIO)
                .openStream();
        try {
            assertEquals(0, stream.requestStart());
            assertEquals(0, stream.requestStop());
        } finally {
            stream.close();
        }
    }

    public void testOpenSlOutputLifecycle() {
        AudioStream stream = new AudioStreamBuilder()
                .setAudioApi(AudioApi.OPENSL_ES)
                .openStream();
        try {
            assertEquals(0, stream.requestStart());
            assertEquals(0, stream.requestStop());
        } finally {
            stream.close();
        }
    }
}
```

- [ ] **Step 2: Run instrumentation tests**

Run:

```bash
cd android/oboe-wrapper
./gradlew :oboe-wrapper:connectedDebugAndroidTest
```

Expected with device/emulator: all smoke tests pass. Expected without device/emulator: Gradle reports no connected devices; record that output and continue only after non-device gates pass.

- [ ] **Step 3: Commit**

```bash
git add android/oboe-wrapper/oboe-wrapper/src/androidTest/java/com/google/oboe/AudioStreamSmokeTest.java
git commit -m "Add Android smoke tests for Rust-native AAudio and OpenSL wrappers" \
  -m "The replacement path is accepted by wrapper-level lifecycle tests rather than C++ GTest compatibility." \
  -m "Constraint: Device or emulator is required for runtime audio verification" \
  -m "Confidence: medium" \
  -m "Scope-risk: narrow" \
  -m "Tested: ./gradlew :oboe-wrapper:connectedDebugAndroidTest" \
  -m "Not-tested: Input stream and callback delivery"
```

## Task 10: Real AAudio And OpenSL FFI Replacement

**Files:**
- Modify: `rust/oboe-android/src/aaudio.rs`
- Modify: `rust/oboe-android/src/opensles.rs`
- Modify: `rust/oboe-android/src/backend.rs`
- Modify: `rust/oboe-core/src/builder.rs`
- Modify: `rust/oboe-core/src/types.rs`

- [ ] **Step 1: Extend backend trait for I/O operations**

Update `rust/oboe-android/src/backend.rs`:

```rust
use oboe_core::builder::StreamBuilder;
use oboe_core::error::Result;
use oboe_core::stream::StreamState;

pub trait AudioBackend {
    fn open(builder: &StreamBuilder) -> Result<Self>
    where
        Self: Sized;
    fn request_start(&mut self) -> Result<()>;
    fn request_stop(&mut self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
    fn state(&self) -> StreamState;
    fn write_f32(&mut self, audio: &[f32], timeout_nanos: i64) -> Result<i32>;
    fn read_f32(&mut self, audio: &mut [f32], timeout_nanos: i64) -> Result<i32>;
}
```

- [ ] **Step 2: Add fake backend I/O tests**

Update `rust/oboe-android/src/fake.rs` to implement `write_f32` and `read_f32`:

```rust
    fn write_f32(&mut self, audio: &[f32], _timeout_nanos: i64) -> Result<i32> {
        Ok(audio.len() as i32)
    }

    fn read_f32(&mut self, audio: &mut [f32], _timeout_nanos: i64) -> Result<i32> {
        for sample in audio.iter_mut() {
            *sample = 0.0;
        }
        Ok(audio.len() as i32)
    }
```

Add a fake test:

```rust
    #[test]
    fn fake_backend_reads_and_writes_float_buffers() {
        let mut backend = FakeBackend::open(&StreamBuilder::default()).unwrap();
        assert_eq!(backend.write_f32(&[0.0, 0.5], 0), Ok(2));
        let mut audio = [1.0, 1.0, 1.0];
        assert_eq!(backend.read_f32(&mut audio, 0), Ok(3));
        assert_eq!(audio, [0.0, 0.0, 0.0]);
    }
```

- [ ] **Step 3: Implement real AAudio calls behind a platform module**

Replace the AAudio skeleton with a backend that stores the raw `AAudioStream` pointer and calls Android symbols for open/start/stop/read/write/close. Keep the public Rust type `AAudioBackend` unchanged so JNI and tests do not change.

The file must contain these FFI declarations:

```rust
#[repr(C)]
struct AAudioStreamBuilder {
    _private: [u8; 0],
}

#[repr(C)]
struct AAudioStream {
    _private: [u8; 0],
}

extern "C" {
    fn AAudio_createStreamBuilder(builder: *mut *mut AAudioStreamBuilder) -> i32;
    fn AAudioStreamBuilder_setDirection(builder: *mut AAudioStreamBuilder, direction: i32);
    fn AAudioStreamBuilder_setFormat(builder: *mut AAudioStreamBuilder, format: i32);
    fn AAudioStreamBuilder_openStream(
        builder: *mut AAudioStreamBuilder,
        stream: *mut *mut AAudioStream,
    ) -> i32;
    fn AAudioStreamBuilder_delete(builder: *mut AAudioStreamBuilder) -> i32;
    fn AAudioStream_requestStart(stream: *mut AAudioStream) -> i32;
    fn AAudioStream_requestStop(stream: *mut AAudioStream) -> i32;
    fn AAudioStream_write(
        stream: *mut AAudioStream,
        buffer: *const core::ffi::c_void,
        num_frames: i32,
        timeout_nanos: i64,
    ) -> i32;
    fn AAudioStream_read(
        stream: *mut AAudioStream,
        buffer: *mut core::ffi::c_void,
        num_frames: i32,
        timeout_nanos: i64,
    ) -> i32;
    fn AAudioStream_close(stream: *mut AAudioStream) -> i32;
}
```

Map `Direction::Output` to `AAUDIO_DIRECTION_OUTPUT`, `Direction::Input` to `AAUDIO_DIRECTION_INPUT`, `Format::Float` to `AAUDIO_FORMAT_PCM_FLOAT`, and `Format::I16` to `AAUDIO_FORMAT_PCM_I16`. Return `Error::InvalidArgument` for unsupported first-phase formats.

- [ ] **Step 4: Implement real OpenSL calls behind a platform module**

Replace the OpenSL skeleton with a backend that owns engine, output mixer, player/recorder, and buffer queue pointers. Keep `OpenSlBackend` name stable.

The file must expose the same trait methods as AAudio. For the first replacement phase, support float buffers by converting to/from i16 internally through `oboe-core/src/format.rs` before queueing buffers when OpenSL requires PCM 16-bit.

- [ ] **Step 5: Verify non-device Rust checks**

Run:

```bash
cargo fmt --manifest-path rust/Cargo.toml -- --check
cargo clippy --manifest-path rust/Cargo.toml --workspace --tests -- -D warnings
cargo test --manifest-path rust/Cargo.toml
```

Expected: fmt, clippy, and tests pass.

- [ ] **Step 6: Verify Android wrapper runtime**

Run:

```powershell
.\tools\build-rust-android.ps1 -AndroidNdk $Env:ANDROID_NDK_HOME -ApiLevel 21
```

Then:

```bash
cd android/oboe-wrapper
./gradlew :oboe-wrapper:connectedDebugAndroidTest
```

Expected: AAudio and OpenSL smoke tests pass on an attached emulator/device.

- [ ] **Step 7: Commit**

```bash
git add rust/oboe-android/src/aaudio.rs rust/oboe-android/src/opensles.rs rust/oboe-android/src/backend.rs rust/oboe-android/src/fake.rs rust/oboe-core/src/builder.rs rust/oboe-core/src/types.rs rust/oboe-core/src/format.rs
git commit -m "Replace Android stream lifecycle with Rust AAudio and OpenSL backends" \
  -m "The first Rust-native phase requires both Android audio APIs to open, start, stop, close, and move audio buffers without src C++ ownership." \
  -m "Constraint: OpenSL float buffers are converted through Rust core format helpers" \
  -m "Rejected: Route real backend calls through src/aaudio or src/opensles | that keeps C++ as the main path" \
  -m "Confidence: medium" \
  -m "Scope-risk: broad" \
  -m "Tested: cargo fmt --manifest-path rust/Cargo.toml -- --check; cargo clippy --manifest-path rust/Cargo.toml --workspace --tests -- -D warnings; cargo test --manifest-path rust/Cargo.toml; ./gradlew :oboe-wrapper:connectedDebugAndroidTest" \
  -m "Not-tested: Long-running callback stress and input recording permission denial"
```

## Task 11: Retire `src` From The Main Build Path

**Files:**
- Modify: `CMakeLists.txt`
- Create: `docs/superpowers/migration/2026-05-02-src-retirement-status.md`

- [ ] **Step 1: Document final `src` retirement status**

Write `docs/superpowers/migration/2026-05-02-src-retirement-status.md`:

```markdown
# src Retirement Status

The Rust-native Android wrapper is the main implementation path.

| `src` subtree | Status | Replacement |
| --- | --- | --- |
| `src/common` | legacy-reference | `rust/oboe-core/src` |
| `src/fifo` | legacy-reference | `rust/oboe-core/src/fifo.rs` |
| `src/flowgraph` | legacy-reference | `rust/oboe-core/src/flowgraph.rs`, `rust/oboe-core/src/resampler.rs` |
| `src/aaudio` | legacy-reference | `rust/oboe-android/src/aaudio.rs` |
| `src/opensles` | legacy-reference | `rust/oboe-android/src/opensles.rs` |
| `src/rust` | legacy-reference | `rust/oboe-jni/src/lib.rs` |

No new feature work should target these C++ implementation files.
```

- [ ] **Step 2: Keep legacy CMake explicitly marked as legacy**

Modify the top of `CMakeLists.txt` after `project(oboe)`:

```cmake
option(OBOE_BUILD_LEGACY_CPP "Build the legacy C++ Oboe target for reference and compatibility checks" OFF)

if(NOT OBOE_BUILD_LEGACY_CPP)
    message(STATUS "Oboe: legacy C++ target is disabled. Use android/oboe-wrapper for the Rust-native main path.")
    return()
endif()
```

- [ ] **Step 3: Verify legacy target is no longer the default main build**

Run:

```bash
cmake -S . -B build-rust-native-default
```

Expected: configure succeeds and prints `legacy C++ target is disabled`.

Run:

```bash
cmake -S . -B build-legacy-cpp -DOBOE_BUILD_LEGACY_CPP=ON
```

Expected: legacy C++ target configures for reference checks.

- [ ] **Step 4: Commit**

```bash
git add CMakeLists.txt docs/superpowers/migration/2026-05-02-src-retirement-status.md
git commit -m "Retire src C++ from the default build path" \
  -m "The approved migration target makes Rust/JNI/Java the main implementation, so CMake must stop presenting src as the default artifact." \
  -m "Constraint: Legacy C++ remains opt-in for reference checks during migration" \
  -m "Confidence: medium" \
  -m "Scope-risk: broad" \
  -m "Tested: cmake -S . -B build-rust-native-default; cmake -S . -B build-legacy-cpp -DOBOE_BUILD_LEGACY_CPP=ON" \
  -m "Not-tested: Existing downstream C++ sample apps"
```

## Task 12: Final Verification Pass

**Files:**
- Modify: `docs/superpowers/plans/2026-05-02-rust-native-oboe-android-replacement.md`

- [ ] **Step 1: Run Rust checks**

Run:

```bash
cargo fmt --manifest-path rust/Cargo.toml -- --check
cargo clippy --manifest-path rust/Cargo.toml --workspace --tests -- -D warnings
cargo test --manifest-path rust/Cargo.toml
```

Expected: all pass.

- [ ] **Step 2: Run Android wrapper checks**

Run:

```powershell
.\tools\build-rust-android.ps1 -AndroidNdk $Env:ANDROID_NDK_HOME -ApiLevel 21
```

Then:

```bash
cd android/oboe-wrapper
./gradlew :oboe-wrapper:assembleDebug
./gradlew :oboe-wrapper:connectedDebugAndroidTest
```

Expected: assemble passes. Connected tests pass when a device or emulator is attached.

- [ ] **Step 3: Run default CMake retirement check**

Run:

```bash
cmake -S . -B build-rust-native-default
```

Expected: configure succeeds and reports that the legacy C++ target is disabled.

- [ ] **Step 4: Record known gaps in the final response**

Final report must include:

```text
Changed files:
- rust/Cargo.toml
- rust/oboe-core/...
- rust/oboe-android/...
- rust/oboe-jni/...
- android/oboe-wrapper/...
- CMakeLists.txt
- docs/superpowers/migration/2026-05-02-src-retirement-status.md

Verification:
- cargo fmt --manifest-path rust/Cargo.toml -- --check
- cargo clippy --manifest-path rust/Cargo.toml --workspace --tests -- -D warnings
- cargo test --manifest-path rust/Cargo.toml
- tools/build-rust-android.ps1
- gradlew assembleDebug
- gradlew connectedDebugAndroidTest, or exact no-device blocker
- cmake default retirement check

Remaining risks:
- Long callback stress
- Input permission denial behavior
- Full sample app migration
```

- [ ] **Step 5: Commit plan checkbox updates if they were modified during execution**

```bash
git add docs/superpowers/plans/2026-05-02-rust-native-oboe-android-replacement.md
git commit -m "Record Rust-native replacement verification status" \
  -m "The execution plan is the handoff record for what was verified and which device-dependent gaps remain." \
  -m "Confidence: medium" \
  -m "Scope-risk: narrow" \
  -m "Tested: Final verification commands listed in the plan" \
  -m "Not-tested: Items explicitly listed as remaining risks"
```

## Self-Review

- Spec coverage: The plan covers Rust crate API, Java wrapper, AAudio, OpenSL ES, JNI, `src` retirement, and Rust/Android wrapper tests.
- Empty-marker scan: The plan has no empty requirement markers, no undefined fill-in steps, and no task that points to another task instead of giving concrete commands.
- Type consistency: `AudioApi`, `StreamBuilder`, `StreamState`, `AudioBackend`, `AAudioBackend`, `OpenSlBackend`, `AudioStreamBuilder`, and `AudioStream` are named consistently across Rust, JNI, and Java.
