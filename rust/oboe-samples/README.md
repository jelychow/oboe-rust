# oboe-samples

Pure Rust sample engines and test contracts derived from Android Oboe examples.

This crate keeps sample audio logic testable without an Android runtime. Android
sample apps can call into this logic through JNI crates in the repository, but
those JNI crates are not published in the alpha release lane.

The sample crate is useful for:

- Verifying rendering and parsing behavior on the host.
- Keeping Oboe sample concepts visible during the Rust-native migration.
- Providing small examples for downstream users once backend APIs stabilize.
