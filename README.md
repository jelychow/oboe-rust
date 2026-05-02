# oboe-rust

[简体中文](README.zh-CN.md)

This repository has been reduced to the Rust-native Android audio path.

The legacy C++ implementation, C++ public headers, CMake/Prefab build scripts, sample apps, and C++ test runner have been removed. The supported implementation is now:

- `rust/oboe-core`: backend-neutral stream, builder, FIFO, format, resampler, callback, and extension state.
- `rust/oboe-android`: Android AAudio and OpenSL ES backend FFI.
- `rust/oboe-jni`: JNI handle layer exposed to Java.
- `android/oboe-wrapper`: Android Java wrapper and smoke tests.
- `tools/build-rust-android.ps1`: Android ABI build helper for `liboboe_jni.so`.
- `tools/build-rust-android.sh`: Linux/macOS Android ABI build helper for GitHub Actions and local publishing.

## Release Scope

The Rust crates are currently alpha-quality. Before publishing or consuming them
as library dependencies, read `docs/rust-oboe-release-scope.md`. The alpha
release is not a drop-in replacement for the C++ Oboe API.

## Build and Test

```sh
cargo fmt --manifest-path rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rust/Cargo.toml --workspace --tests -- -D warnings
cargo test --manifest-path rust/Cargo.toml
```

## Rust Alpha Release Check

Run the release gate before publishing Rust crates:

```sh
tools/check-rust-release.sh
```

Include Android target checks for the publishable crates when the Rust Android
targets are installed:

```sh
CHECK_ANDROID_ABI=1 tools/check-rust-release.sh
```

For the first crates.io release, `oboe-core` must be published and visible in
the registry before dependent crate dry-runs can resolve it. After `oboe-core`
is indexed, run the dependent dry-runs:

```sh
VERIFY_PUBLISHED_DEPS=1 tools/check-rust-release.sh
```

JNI `.so` builds for Android sample apps are smoke checks rather than crates.io
publish gates. Use `docs/rust-android-device-smoke.md` for that path.

## GitHub Packages

The Android wrapper can be published as an AAR to GitHub Packages:

```groovy
repositories {
    maven {
        url = uri("https://maven.pkg.github.com/jelychow/oboe-rust")
        credentials {
            username = findProperty("gpr.user") ?: System.getenv("GITHUB_ACTOR")
            password = findProperty("gpr.key") ?: System.getenv("GITHUB_TOKEN")
        }
    }
}

dependencies {
    implementation("io.github.jelychow.oboe:oboe-rust-android:0.1.0-alpha.1")
}
```

Build the native JNI libraries before publishing so the AAR contains the
Android `.so` files:

```sh
RUST_ANDROID_LIBRARIES=oboe-jni tools/build-rust-android.sh
```

Publish locally for validation:

```sh
cd android/oboe-wrapper
./gradlew :oboe-wrapper:publishReleasePublicationToMavenLocal
```

Publish to GitHub Packages with a token that can write packages:

```sh
cd android/oboe-wrapper
GITHUB_ACTOR=<github-user> GITHUB_TOKEN=<token> \
  ./gradlew :oboe-wrapper:publishReleasePublicationToGitHubPackagesRepository
```

The repository also includes `.github/workflows/publish-github-packages.yml`,
which publishes the Android wrapper package from a release, a `v*` tag, or a
manual workflow dispatch.

## Android Gradle Sync

Open the repository root in Android Studio. The root Gradle project exposes:

- `:oboe-wrapper`: Android library module for Java/JNI consumers.
- `:oboe-smoke-app`: installable smoke app using `implementation project(':oboe-wrapper')`.

To compile the Java wrapper without Gradle:

```powershell
javac.exe -Xlint:all -d build\javac-oboe-wrapper android\oboe-wrapper\oboe-wrapper\src\main\java\com\google\oboe\*.java
```

To build Android JNI libraries, provide an Android NDK path. The Rust AAudio backend
links against `libaaudio`, so Android API 26 is the default native build baseline.

```powershell
.\tools\build-rust-android.ps1 -AndroidNdk C:\path\to\Android\Sdk\ndk\<version>
```

To build a signed smoke-test APK without Gradle:

```powershell
.\tools\build-smoke-apk.ps1 -AndroidSdk C:\path\to\Android\Sdk
```

To install it on a connected device or emulator:

```powershell
.\tools\build-smoke-apk.ps1 -AndroidSdk C:\path\to\Android\Sdk -Install
```
