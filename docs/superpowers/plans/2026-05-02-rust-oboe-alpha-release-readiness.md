# Rust Oboe Alpha Release Readiness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prepare the current Rust-native Oboe workspace for an honest `0.1.0-alpha.1` release by fixing Cargo publish blockers, documenting the C++ parity gap, and adding repeatable release checks.

**Architecture:** Treat this as an alpha release, not a full C++ Oboe replacement. Publish only the reusable Rust crates (`oboe-core`, `oboe-android`, `oboe-samples`) and keep JNI/app helper crates private until AAR/JNI distribution is designed. Add small source-level guardrails so docs, support status, backend error mapping, and Cargo publish checks do not drift silently.

**Tech Stack:** Rust 2021, Cargo workspace metadata, Android AAudio/OpenSL ES FFI, Java wrapper smoke path, shell release scripts, crates.io dry-run checks.

---

## Scope Check

Full C++ Oboe parity is too large for one implementation plan. This plan covers the alpha release gate:

- Cargo metadata and publishability.
- Clear public scope and non-parity documentation.
- A small `oboe-core` capability matrix exposed in code and docs.
- Better AAudio error preservation.
- Repeatable host and Android release checks.

The following require separate implementation plans after this alpha release gate:

- Full `AudioStreamBuilder` parity: usage, content type, input preset, session id, device id, channel mask, capture policy, privacy, spatialization, package name, attribution tag, and conversion policy.
- Real callback engine: Rust callbacks, Java callbacks, error callbacks, presentation callbacks, and routing callbacks driven by native audio thread events.
- Runtime diagnostics: timestamp, latency, xrun count, buffer size tuning, frames read/written, available frames, wait-for-state-change, pause, flush, and release.
- Android distribution: Maven/AAR packaging, Prefab-equivalent layout, and ABI artifact signing.

## File Structure

- Modify `rust/Cargo.toml`: workspace package metadata shared by crates.
- Modify `rust/oboe-core/Cargo.toml`: alpha package metadata and README reference.
- Modify `rust/oboe-android/Cargo.toml`: alpha package metadata plus versioned path dependency on `oboe-core`.
- Modify `rust/oboe-samples/Cargo.toml`: alpha package metadata plus versioned path dependency on `oboe-core`.
- Modify `rust/oboe-jni/Cargo.toml`, `rust/oboe-samples-jni/Cargo.toml`, `rust/minimaloboe-rust-jni/Cargo.toml`, `rust/openai-realtime-jni/Cargo.toml`: mark non-publishable for this release lane.
- Create `rust/oboe-core/README.md`, `rust/oboe-android/README.md`, `rust/oboe-samples/README.md`: crate-specific crates.io entry docs.
- Create `docs/rust-oboe-release-scope.md`: human release scope and C++ comparison.
- Create `rust/oboe-core/src/capabilities.rs`: source-of-truth capability status table.
- Modify `rust/oboe-core/src/lib.rs`: export `capabilities`.
- Modify `rust/oboe-core/src/error.rs`: preserve native platform error codes.
- Modify `rust/oboe-android/src/aaudio.rs`: map negative AAudio results to platform error codes.
- Create `tools/check-rust-release.sh`: repeatable release verification command.
- Modify `README.md` and `docs/README.md`: point users to alpha release docs and checks.

---

### Task 1: Cargo Release Metadata And Publish Boundaries

**Files:**
- Modify: `rust/Cargo.toml`
- Modify: `rust/oboe-core/Cargo.toml`
- Modify: `rust/oboe-android/Cargo.toml`
- Modify: `rust/oboe-samples/Cargo.toml`
- Modify: `rust/oboe-jni/Cargo.toml`
- Modify: `rust/oboe-samples-jni/Cargo.toml`
- Modify: `rust/minimaloboe-rust-jni/Cargo.toml`
- Modify: `rust/openai-realtime-jni/Cargo.toml`

- [ ] **Step 1: Reproduce the current publish blocker**

Run:

```bash
cargo publish --dry-run --manifest-path rust/Cargo.toml -p oboe-android --allow-dirty
```

Expected: FAIL with this message:

```text
all dependencies must have a version requirement specified when publishing.
dependency `oboe-core` does not specify a version
```

- [ ] **Step 2: Update workspace package metadata**

Replace the `[workspace.package]` section in `rust/Cargo.toml` with:

```toml
[workspace.package]
edition = "2021"
license = "Apache-2.0"
version = "0.1.0-alpha.1"
repository = "https://github.com/jelychow/oboe-rust"
homepage = "https://github.com/jelychow/oboe-rust"
documentation = "https://docs.rs/oboe-core"
rust-version = "1.82"
```

- [ ] **Step 3: Update `oboe-core` package metadata**

Replace `rust/oboe-core/Cargo.toml` with:

```toml
[package]
name = "oboe-core"
edition.workspace = true
license.workspace = true
version.workspace = true
repository.workspace = true
homepage.workspace = true
documentation.workspace = true
rust-version.workspace = true
description = "Backend-neutral Rust core types for an experimental Rust-native Oboe audio API."
readme = "README.md"
keywords = ["android", "audio", "oboe"]
categories = ["multimedia::audio", "api-bindings"]

[lib]
crate-type = ["rlib"]
```

- [ ] **Step 4: Update `oboe-android` package metadata and dependency**

Replace `rust/oboe-android/Cargo.toml` with:

```toml
[package]
name = "oboe-android"
edition.workspace = true
license.workspace = true
version.workspace = true
repository.workspace = true
homepage.workspace = true
documentation = "https://docs.rs/oboe-android"
rust-version.workspace = true
description = "Experimental Rust-native Android AAudio and OpenSL ES backends for Oboe-style audio streams."
readme = "README.md"
keywords = ["android", "aaudio", "opensles", "audio"]
categories = ["multimedia::audio", "api-bindings", "os::android-apis"]

[dependencies]
oboe-core = { version = "0.1.0-alpha.1", path = "../oboe-core" }

[lib]
crate-type = ["rlib"]
```

- [ ] **Step 5: Update `oboe-samples` package metadata and dependency**

Replace `rust/oboe-samples/Cargo.toml` with:

```toml
[package]
name = "oboe-samples"
edition.workspace = true
license.workspace = true
version.workspace = true
repository.workspace = true
homepage.workspace = true
documentation = "https://docs.rs/oboe-samples"
rust-version.workspace = true
description = "Pure Rust sample audio engines and contracts derived from Android Oboe examples."
readme = "README.md"
keywords = ["android", "audio", "samples"]
categories = ["multimedia::audio"]

[dependencies]
oboe-core = { version = "0.1.0-alpha.1", path = "../oboe-core" }

[lib]
crate-type = ["rlib"]
```

- [ ] **Step 6: Mark JNI and app helper crates private for this release lane**

Add `publish = false` under `[package]` in:

```text
rust/oboe-jni/Cargo.toml
rust/oboe-samples-jni/Cargo.toml
rust/minimaloboe-rust-jni/Cargo.toml
rust/openai-realtime-jni/Cargo.toml
```

For `rust/oboe-jni/Cargo.toml`, the top of the file should become:

```toml
[package]
name = "oboe-jni"
edition.workspace = true
license.workspace = true
version.workspace = true
publish = false
```

For `rust/oboe-samples-jni/Cargo.toml`, the top of the file should become:

```toml
[package]
name = "oboe-samples-jni"
edition.workspace = true
license.workspace = true
version.workspace = true
publish = false
```

For `rust/minimaloboe-rust-jni/Cargo.toml`, the top of the file should become:

```toml
[package]
name = "minimaloboe-rust-jni"
edition.workspace = true
license.workspace = true
version.workspace = true
publish = false
```

For `rust/openai-realtime-jni/Cargo.toml`, the top of the file should become:

```toml
[package]
name = "openai-realtime-jni"
edition.workspace = true
license.workspace = true
version.workspace = true
publish = false
```

- [ ] **Step 7: Run publish dry-runs for publishable crates**

Run:

```bash
cargo publish --dry-run --manifest-path rust/Cargo.toml -p oboe-core --allow-dirty
cargo publish --dry-run --manifest-path rust/Cargo.toml -p oboe-android --allow-dirty
cargo publish --dry-run --manifest-path rust/Cargo.toml -p oboe-samples --allow-dirty
```

Expected: each command reaches `warning: aborting upload due to dry run` and does not fail manifest validation.

- [ ] **Step 8: Commit**

Run:

```bash
git add rust/Cargo.toml rust/oboe-core/Cargo.toml rust/oboe-android/Cargo.toml rust/oboe-samples/Cargo.toml rust/oboe-jni/Cargo.toml rust/oboe-samples-jni/Cargo.toml rust/minimaloboe-rust-jni/Cargo.toml rust/openai-realtime-jni/Cargo.toml
git commit -m "Prepare Rust crates for alpha publication

The reusable Rust crates now carry crates.io metadata and versioned local dependencies, while JNI and application helper crates remain private until their Android artifact story is designed.

Constraint: crates.io rejects path-only dependencies during publication
Rejected: Publish JNI cdylib crates immediately | JNI crates need AAR and ABI packaging decisions first
Confidence: high
Scope-risk: narrow
Directive: Do not publish JNI helper crates until Maven or AAR distribution is explicitly planned
Tested: cargo publish dry-run for oboe-core, oboe-android, and oboe-samples
Not-tested: Real crates.io upload"
```

---

### Task 2: Crate README And Release Scope Documentation

**Files:**
- Create: `rust/oboe-core/README.md`
- Create: `rust/oboe-android/README.md`
- Create: `rust/oboe-samples/README.md`
- Create: `docs/rust-oboe-release-scope.md`
- Modify: `README.md`
- Modify: `docs/README.md`

- [ ] **Step 1: Create `oboe-core` README**

Create `rust/oboe-core/README.md` with:

```markdown
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
```

- [ ] **Step 2: Create `oboe-android` README**

Create `rust/oboe-android/README.md` with:

```markdown
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
```

- [ ] **Step 3: Create `oboe-samples` README**

Create `rust/oboe-samples/README.md` with:

```markdown
# oboe-samples

Pure Rust sample engines and test contracts derived from Android Oboe examples.

This crate keeps sample audio logic testable without an Android runtime. Android
sample apps can call into this logic through JNI crates in the repository, but
those JNI crates are not published in the alpha release lane.

The sample crate is useful for:

- Verifying rendering and parsing behavior on the host.
- Keeping Oboe sample concepts visible during the Rust-native migration.
- Providing small examples for downstream users once backend APIs stabilize.
```

- [ ] **Step 4: Create release scope document**

Create `docs/rust-oboe-release-scope.md` with:

```markdown
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
| `oboe-samples-jni` | No | Android sample bridge follows the JNI release lane. |
| `minimaloboe-rust-jni` | No | Demo app helper crate. |
| `openai-realtime-jni` | No | Product sample helper crate, not an Oboe library crate. |

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
```

- [ ] **Step 5: Update root README**

In `README.md`, add this paragraph after the title and before `Build and Test`:

```markdown
## Release Scope

The Rust crates are currently alpha-quality. Before publishing or consuming them
as library dependencies, read `docs/rust-oboe-release-scope.md`. The alpha
release is not a drop-in replacement for the C++ Oboe API.
```

- [ ] **Step 6: Update docs index**

In `docs/README.md`, add this bullet under `Active Paths`:

```markdown
- `rust-oboe-release-scope.md`: alpha release boundaries and C++ Oboe parity snapshot.
```

- [ ] **Step 7: Verify docs references**

Run:

```bash
rg -n "not a drop-in replacement|0.1.0-alpha.1|C\\+\\+ Oboe Parity Snapshot" README.md docs/rust-oboe-release-scope.md rust/oboe-core/README.md rust/oboe-android/README.md rust/oboe-samples/README.md
```

Expected: output includes matches in the new README files and `docs/rust-oboe-release-scope.md`.

- [ ] **Step 8: Commit**

Run:

```bash
git add README.md docs/README.md docs/rust-oboe-release-scope.md rust/oboe-core/README.md rust/oboe-android/README.md rust/oboe-samples/README.md
git commit -m "Document Rust Oboe alpha release boundaries

The release docs now state which crates are publishable, which JNI crates remain private, and where the Rust alpha differs from C++ Oboe.

Constraint: Current Rust API is not C++ Oboe parity
Rejected: Market alpha crates as a full replacement | unsupported callbacks and diagnostics would mislead users
Confidence: high
Scope-risk: narrow
Directive: Keep release docs synchronized with capability metadata before publishing
Tested: rg verification for alpha scope and parity wording
Not-tested: External rendered docs.rs output"
```

---

### Task 3: Source-Level Capability Metadata

**Files:**
- Create: `rust/oboe-core/src/capabilities.rs`
- Modify: `rust/oboe-core/src/lib.rs`
- Test: `rust/oboe-core/src/lib.rs`
- Test: `rust/oboe-core/src/capabilities.rs`

- [ ] **Step 1: Add a failing public API test in `lib.rs`**

Append this test module to `rust/oboe-core/src/lib.rs`:

```rust
#[cfg(test)]
mod capability_api_tests {
    use super::capabilities::{capability, SupportLevel};

    #[test]
    fn public_capability_api_reports_callback_gap() {
        let capability = capability("stream_callbacks").unwrap();
        assert_eq!(capability.support, SupportLevel::Unsupported);
    }
}
```

- [ ] **Step 2: Run the public API test to verify it fails**

Run:

```bash
cargo test --manifest-path rust/Cargo.toml -p oboe-core public_capability_api_reports_callback_gap
```

Expected: FAIL with this unresolved import:

```text
could not find `capabilities` in the crate root
```

- [ ] **Step 3: Write the capability metadata module with tests**

Create `rust/oboe-core/src/capabilities.rs` with:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupportLevel {
    Supported,
    Partial,
    Unsupported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capability {
    pub name: &'static str,
    pub support: SupportLevel,
    pub note: &'static str,
}

pub const CAPABILITIES: &[Capability] = &[
    Capability {
        name: "aaudio_blocking_io",
        support: SupportLevel::Supported,
        note: "AAudio open/start/stop/close plus blocking f32 read and write are available.",
    },
    Capability {
        name: "opensles_output",
        support: SupportLevel::Partial,
        note: "OpenSL ES output enqueue is available; input recording is not implemented.",
    },
    Capability {
        name: "stream_callbacks",
        support: SupportLevel::Unsupported,
        note: "Callback flags can be stored, but native audio callback dispatch is not available.",
    },
    Capability {
        name: "latency_and_xrun_diagnostics",
        support: SupportLevel::Unsupported,
        note: "Timestamp, xrun count, latency, and buffer tuning APIs are not available.",
    },
    Capability {
        name: "advanced_builder_fields",
        support: SupportLevel::Unsupported,
        note: "Usage, content type, input preset, session id, device id, capture policy, privacy, spatialization, package name, attribution tag, and conversion policy are not available.",
    },
];

pub fn capability(name: &str) -> Option<&'static Capability> {
    CAPABILITIES.iter().find(|capability| capability.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_supported_aaudio_blocking_io() {
        let capability = capability("aaudio_blocking_io").unwrap();
        assert_eq!(capability.support, SupportLevel::Supported);
        assert!(capability.note.contains("blocking f32 read and write"));
    }

    #[test]
    fn tracks_unsupported_callback_dispatch() {
        let capability = capability("stream_callbacks").unwrap();
        assert_eq!(capability.support, SupportLevel::Unsupported);
        assert!(capability.note.contains("not available"));
    }

    #[test]
    fn all_capability_names_are_unique() {
        for (index, capability) in CAPABILITIES.iter().enumerate() {
            assert!(
                CAPABILITIES[index + 1..]
                    .iter()
                    .all(|other| other.name != capability.name),
                "duplicate capability name: {}",
                capability.name
            );
        }
    }
}
```

- [ ] **Step 4: Export the module**

Add this line to `rust/oboe-core/src/lib.rs`:

```rust
pub mod capabilities;
```

The top of `rust/oboe-core/src/lib.rs` should be:

```rust
#![deny(unsafe_op_in_unsafe_fn)]

pub mod builder;
pub mod capabilities;
pub mod error;
pub mod extensions;
pub mod fifo;
pub mod format;
pub mod resampler;
pub mod stream;
pub mod types;

pub const VERSION_CODE: i32 = 1;

#[cfg(test)]
mod capability_api_tests {
    use super::capabilities::{capability, SupportLevel};

    #[test]
    fn public_capability_api_reports_callback_gap() {
        let capability = capability("stream_callbacks").unwrap();
        assert_eq!(capability.support, SupportLevel::Unsupported);
    }
}
```

- [ ] **Step 5: Verify capability tests pass**

Run:

```bash
cargo test --manifest-path rust/Cargo.toml -p oboe-core capabilities::
cargo test --manifest-path rust/Cargo.toml -p oboe-core public_capability_api_reports_callback_gap
```

Expected: PASS with 3 capability module tests and 1 public API test.

- [ ] **Step 6: Commit**

Run:

```bash
git add rust/oboe-core/src/lib.rs rust/oboe-core/src/capabilities.rs
git commit -m "Expose alpha capability metadata in oboe-core

The Rust core now exposes a small capability table that downstream docs and users can inspect to understand supported, partial, and unsupported alpha features.

Constraint: Rust alpha is not C++ Oboe parity
Rejected: Keep capability status only in prose | source-level metadata is easier to test for drift
Confidence: medium
Scope-risk: narrow
Directive: Update CAPABILITIES whenever public support status changes
Tested: cargo test -p oboe-core capabilities::
Not-tested: docs.rs rendered capability documentation"
```

---

### Task 4: Preserve Native AAudio Error Codes

**Files:**
- Modify: `rust/oboe-core/src/error.rs`
- Modify: `rust/oboe-android/src/aaudio.rs`
- Test: `rust/oboe-core/src/error.rs`
- Test: `rust/oboe-android/src/aaudio.rs`

- [ ] **Step 1: Add failing error mapping tests**

Append these tests to `rust/oboe-core/src/error.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_negative_platform_result_to_platform_error() {
        assert_eq!(Error::from_platform_result(-899), Error::Platform(-899));
        assert_eq!(Error::from_platform_result(-1), Error::Platform(-1));
    }

    #[test]
    fn maps_non_negative_platform_result_to_internal() {
        assert_eq!(Error::from_platform_result(0), Error::Internal);
        assert_eq!(Error::from_platform_result(7), Error::Internal);
    }
}
```

Run:

```bash
cargo test --manifest-path rust/Cargo.toml -p oboe-core error::
```

Expected: FAIL with `no variant or associated item named 'Platform'` or `from_platform_result` missing.

- [ ] **Step 2: Implement platform error preservation**

Replace `rust/oboe-core/src/error.rs` with:

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
    Platform(i32),
}

impl Error {
    pub fn from_platform_result(result: i32) -> Self {
        if result < 0 {
            Self::Platform(result)
        } else {
            Self::Internal
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_negative_platform_result_to_platform_error() {
        assert_eq!(Error::from_platform_result(-899), Error::Platform(-899));
        assert_eq!(Error::from_platform_result(-1), Error::Platform(-1));
    }

    #[test]
    fn maps_non_negative_platform_result_to_internal() {
        assert_eq!(Error::from_platform_result(0), Error::Internal);
        assert_eq!(Error::from_platform_result(7), Error::Internal);
    }
}
```

- [ ] **Step 3: Map negative AAudio read/write results to `Error::Platform`**

In `rust/oboe-android/src/aaudio.rs`, replace this code in `write_raw`:

```rust
if result < 0 {
    Err(Error::InvalidState)
} else {
    Ok(result)
}
```

with:

```rust
if result < 0 {
    Err(Error::from_platform_result(result))
} else {
    Ok(result)
}
```

Do the same replacement in `read_raw`.

- [ ] **Step 4: Add host test for platform error mapping helper behavior**

Append this test to the existing `#[cfg(test)] mod tests` in `rust/oboe-android/src/aaudio.rs`:

```rust
#[test]
fn negative_platform_results_preserve_native_code() {
    assert_eq!(Error::from_platform_result(-899), Error::Platform(-899));
}
```

- [ ] **Step 5: Verify tests pass**

Run:

```bash
cargo test --manifest-path rust/Cargo.toml -p oboe-core error::
cargo test --manifest-path rust/Cargo.toml -p oboe-android negative_platform_results_preserve_native_code
```

Expected: both commands PASS.

- [ ] **Step 6: Commit**

Run:

```bash
git add rust/oboe-core/src/error.rs rust/oboe-android/src/aaudio.rs
git commit -m "Preserve native platform error codes

AAudio failures now retain the negative platform result instead of collapsing every backend failure into InvalidState, which makes alpha device failures diagnosable.

Constraint: Android audio failures are device-specific and need native result codes for triage
Rejected: Keep coarse InvalidState mapping | it hides actionable AAudio failure codes
Confidence: medium
Scope-risk: moderate
Directive: Preserve platform codes when adding more Android FFI calls
Tested: cargo test -p oboe-core error::; cargo test -p oboe-android negative_platform_results_preserve_native_code
Not-tested: Physical-device AAudio failure injection"
```

---

### Task 5: Repeatable Rust Release Check Script

**Files:**
- Create: `tools/check-rust-release.sh`
- Modify: `README.md`

- [ ] **Step 1: Prove the release check script is missing**

Run:

```bash
tools/check-rust-release.sh
```

Expected: FAIL with `No such file or directory`.

- [ ] **Step 2: Create the release check script**

Create `tools/check-rust-release.sh` with:

```bash
#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
manifest="$repo_root/rust/Cargo.toml"

cargo fmt --manifest-path "$manifest" --all -- --check
cargo clippy --manifest-path "$manifest" --workspace --tests -- -D warnings
cargo test --manifest-path "$manifest"

publishable_crates=(
  oboe-core
  oboe-android
  oboe-samples
)

for crate in "${publishable_crates[@]}"; do
  cargo publish --dry-run --manifest-path "$manifest" -p "$crate" --allow-dirty
done

if [[ "${CHECK_ANDROID_ABI:-0}" == "1" ]]; then
  "$repo_root/tools/build-rust-android.sh"
else
  echo "Skipping Android ABI build. Set CHECK_ANDROID_ABI=1 to include tools/build-rust-android.sh."
fi
```

- [ ] **Step 3: Make the script executable**

Run:

```bash
chmod +x tools/check-rust-release.sh
```

- [ ] **Step 4: Add README release-check command**

Add this section to `README.md` after the existing build commands:

````markdown
## Rust Alpha Release Check

Run the release gate before publishing Rust crates:

```sh
tools/check-rust-release.sh
```

Include Android ABI builds when an Android NDK is available:

```sh
ANDROID_NDK=/path/to/Android/Sdk/ndk/<version> CHECK_ANDROID_ABI=1 tools/check-rust-release.sh
```
````

- [ ] **Step 5: Run host release check**

Run:

```bash
tools/check-rust-release.sh
```

Expected:

```text
Skipping Android ABI build. Set CHECK_ANDROID_ABI=1 to include tools/build-rust-android.sh.
```

and no Cargo command fails.

- [ ] **Step 6: Commit**

Run:

```bash
git add README.md tools/check-rust-release.sh
git commit -m "Add repeatable Rust alpha release check

The release script runs formatting, clippy, tests, and Cargo dry-runs for publishable Rust crates, with an opt-in Android ABI build for machines with an NDK.

Constraint: Release verification currently requires several manual commands
Rejected: Require every contributor to remember the command sequence | drift would make publishing fragile
Confidence: high
Scope-risk: narrow
Directive: Keep this script aligned with the set of publishable crates
Tested: tools/check-rust-release.sh
Not-tested: CHECK_ANDROID_ABI=1 on every supported host"
```

---

### Task 6: Android ABI And Device Smoke Runbook

**Files:**
- Create: `docs/rust-android-device-smoke.md`
- Modify: `docs/README.md`

- [ ] **Step 1: Create the Android smoke runbook**

Create `docs/rust-android-device-smoke.md` with:

````markdown
# Rust Android Device Smoke Runbook

Use this runbook before publishing or tagging a Rust Oboe alpha release.

## Preconditions

- Android SDK platform tools are installed.
- Android NDK is installed.
- At least one emulator or physical Android device is visible from `adb devices`.
- The repository-local Rust Android targets are installed for the ABIs being built.

## Build Native Libraries

```sh
ANDROID_NDK=/path/to/Android/Sdk/ndk/<version> tools/build-rust-android.sh
```

Expected output includes release builds for these Android ABIs:

```text
aarch64-linux-android
armv7-linux-androideabi
i686-linux-android
x86_64-linux-android
```

## Build Smoke APK

```sh
cd android/oboe-wrapper
ANDROID_USER_HOME=$HOME/.android \
JAVA_HOME=/path/to/jdk-17 \
PATH=/path/to/jdk-17/bin:$PATH \
./gradlew :oboe-smoke-app:assembleDebug --console=plain --no-daemon
```

Expected:

```text
BUILD SUCCESSFUL
```

## Install And Launch

```sh
adb devices
adb -s <device-serial> install -r android/oboe-wrapper/oboe-smoke-app/build/outputs/apk/debug/oboe-smoke-app-debug.apk
adb -s <device-serial> shell am start -n com.google.oboe.smoke/.MainActivity
```

Expected:

```text
Success
Starting: Intent { cmp=com.google.oboe.smoke/.MainActivity }
```

## Logcat Checks

```sh
adb -s <device-serial> logcat -d -t 500 | rg "FATAL EXCEPTION|UnsatisfiedLinkError|oboe|AAudio"
```

Release-blocking failures:

- `FATAL EXCEPTION`
- `UnsatisfiedLinkError`
- Native library missing for the installed ABI
- AAudio stream open failure in the smoke app
````

- [ ] **Step 2: Link the runbook from docs index**

Add this bullet to `docs/README.md` under `Active Paths`:

```markdown
- `rust-android-device-smoke.md`: Android ABI, APK, install, and logcat smoke runbook.
```

- [ ] **Step 3: Verify runbook references**

Run:

```bash
rg -n "Rust Android Device Smoke Runbook|tools/build-rust-android.sh|UnsatisfiedLinkError" docs/rust-android-device-smoke.md docs/README.md
```

Expected: output includes the runbook title, native build command, and logcat failure strings.

- [ ] **Step 4: Commit**

Run:

```bash
git add docs/README.md docs/rust-android-device-smoke.md
git commit -m "Document Android device smoke checks for Rust Oboe

The runbook captures the native ABI build, smoke APK build, install, launch, and logcat checks needed before publishing or tagging an alpha release.

Constraint: Host tests cannot prove Android device loading and AAudio behavior
Rejected: Treat cargo tests as sufficient release proof | JNI and ABI packaging can still fail on device
Confidence: high
Scope-risk: narrow
Directive: Run this runbook on at least one emulator and one physical device before non-alpha release
Tested: rg verification for runbook commands and failure strings
Not-tested: Device smoke execution in this documentation-only task"
```

---

### Task 7: Final Verification And Release Notes

**Files:**
- Create: `docs/rust-alpha-release-checklist.md`

- [ ] **Step 1: Create release checklist**

Create `docs/rust-alpha-release-checklist.md` with:

````markdown
# Rust Oboe Alpha Release Checklist

Use this checklist for `0.1.0-alpha.1`.

## Required Checks

- `cargo fmt --manifest-path rust/Cargo.toml --all -- --check`
- `cargo clippy --manifest-path rust/Cargo.toml --workspace --tests -- -D warnings`
- `cargo test --manifest-path rust/Cargo.toml`
- `cargo publish --dry-run --manifest-path rust/Cargo.toml -p oboe-core --allow-dirty`
- `cargo publish --dry-run --manifest-path rust/Cargo.toml -p oboe-android --allow-dirty`
- `cargo publish --dry-run --manifest-path rust/Cargo.toml -p oboe-samples --allow-dirty`
- `ANDROID_NDK=/path/to/Android/Sdk/ndk/<version> CHECK_ANDROID_ABI=1 tools/check-rust-release.sh`
- Follow `docs/rust-android-device-smoke.md` on an emulator or physical device.

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
````

- [ ] **Step 2: Run final host release gate**

Run:

```bash
tools/check-rust-release.sh
```

Expected: all Cargo checks pass and the script prints the Android ABI skip message unless `CHECK_ANDROID_ABI=1` is set.

- [ ] **Step 3: Run Android ABI release gate when NDK is available**

Run:

```bash
ANDROID_NDK=/path/to/Android/Sdk/ndk/<version> CHECK_ANDROID_ABI=1 tools/check-rust-release.sh
```

Expected: all Cargo checks pass and `tools/build-rust-android.sh` completes all four Android ABIs.

- [ ] **Step 4: Commit**

Run:

```bash
git add docs/rust-alpha-release-checklist.md
git commit -m "Add Rust Oboe alpha release checklist

The checklist records the exact release checks, publish order, and alpha release note wording needed for a controlled crates.io release.

Constraint: Alpha release needs a repeatable human gate, not only scripts
Rejected: Keep release notes implicit | unsupported C++ parity would be easy to overstate
Confidence: high
Scope-risk: narrow
Directive: Update this checklist for every alpha version change
Tested: tools/check-rust-release.sh; optional CHECK_ANDROID_ABI=1 when NDK is available
Not-tested: Actual crates.io publication"
```

---

## Self-Review

**Spec coverage:** The plan covers the previously identified release blockers: Cargo metadata, versioned dependencies, private JNI crates, support-scope docs, C++ parity gap, source-level capability status, native error diagnostics, release checks, and Android smoke guidance.

**Placeholder scan:** This plan uses concrete file paths, concrete code snippets, exact commands, and expected outputs. It avoids open-ended implementation instructions.

**Type consistency:** The plan defines `SupportLevel`, `Capability`, `CAPABILITIES`, `capability`, `Error::Platform(i32)`, and `Error::from_platform_result(i32)` before later tasks reference them. Cargo versions consistently use `0.1.0-alpha.1`.
