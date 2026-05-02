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
