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

GitHub Packages requires authentication when installing packages. For Gradle,
provide `gpr.user` and `gpr.key` in `~/.gradle/gradle.properties`, or use
`GITHUB_ACTOR` and `GITHUB_TOKEN` in CI:

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

For Apache Maven consumers, configure the GitHub Packages repository and server
credentials in `~/.m2/settings.xml`, then declare the AAR dependency with
`<type>aar</type>`:

```xml
<repositories>
  <repository>
    <id>github</id>
    <url>https://maven.pkg.github.com/jelychow/oboe-rust</url>
  </repository>
</repositories>

<dependency>
  <groupId>io.github.jelychow.oboe</groupId>
  <artifactId>oboe-rust-android</artifactId>
  <version>0.1.0-alpha.1</version>
  <type>aar</type>
</dependency>
```

```xml
<settings>
  <servers>
    <server>
      <id>github</id>
      <username>${env.GITHUB_ACTOR}</username>
      <password>${env.GITHUB_TOKEN}</password>
    </server>
  </servers>
</settings>
```

Build the native JNI libraries before publishing so the AAR contains the
Android `.so` files:

```sh
RUST_ANDROID_LIBRARIES=oboe-jni tools/build-rust-android.sh
```

Publish locally for validation. The release workflow runs the same Maven Local
publication before it uploads to GitHub Packages:

```sh
cd android/oboe-wrapper
./gradlew :oboe-wrapper:publishReleasePublicationToMavenLocal \
  -PoboeRust.version=0.1.0-alpha.1
```

Publish to GitHub Packages with a token that can write packages:

```sh
cd android/oboe-wrapper
GITHUB_ACTOR=<github-user> GITHUB_TOKEN=<token> \
  ./gradlew :oboe-wrapper:publishReleasePublicationToGitHubPackagesRepository \
  -PoboeRust.version=0.1.0-alpha.1
```

The repository also includes `.github/workflows/publish-github-packages.yml`,
which verifies and publishes the Android wrapper package automatically when a
GitHub Release is published. Use release tags such as `v0.1.0-alpha.1`; the
workflow strips the leading `v` and publishes package version `0.1.0-alpha.1`.
After publishing, the workflow compiles a temporary Android app that consumes
the package from GitHub Packages and checks that all four `liboboe_jni.so`
ABIs are present. Manual workflow dispatch remains available for retrying a
version explicitly.

## JitPack

JitPack can build the Android wrapper from the public GitHub repository without
GitHub Packages credentials. Add JitPack as a repository:

```groovy
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
        maven { url = uri("https://jitpack.io") }
    }
}
```

Then depend on the wrapper module:

```groovy
dependencies {
    implementation("com.github.jelychow.oboe-rust:oboe-wrapper:<tag-or-commit>")
}
```

For example, use `main-SNAPSHOT` to test the latest `main` branch build, or
create a new Git tag after this JitPack configuration is merged and use that
tag as the version. Older upstream tags predate this Rust/JitPack publishing
script and should not be used for this package.

The root `jitpack.yml` runs `tools/publish-jitpack-android.sh`. The script
installs Rust targets and the Android NDK if needed, builds only
`liboboe_jni.so`, and publishes the Android wrapper to Maven Local with
JitPack's multi-module coordinates:

```sh
JITPACK_GROUP_ID=com.github.jelychow.oboe-rust \
JITPACK_ARTIFACT_ID=oboe-wrapper \
JITPACK_VERSION=main-SNAPSHOT \
  tools/publish-jitpack-android.sh
```

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
