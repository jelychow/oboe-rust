# oboe-rust

[English](README.md)

这个仓库已经收敛为 Rust 原生的 Android 音频实现路径。

旧的 C++ 实现、C++ 公共头文件、CMake/Prefab 构建脚本、示例应用和 C++ 测试入口已经移除。当前支持的实现包括：

- `rust/oboe-core`：与后端无关的 stream、builder、FIFO、format、resampler、callback 和扩展状态。
- `rust/oboe-android`：Android AAudio 和 OpenSL ES 后端 FFI。
- `rust/oboe-jni`：暴露给 Java 的 JNI handle 层。
- `android/oboe-wrapper`：Android Java wrapper 和 smoke test。
- `tools/build-rust-android.ps1`：Windows 上构建 Android ABI `liboboe_jni.so` 的辅助脚本。
- `tools/build-rust-android.sh`：Linux/macOS 上构建 Android ABI 的辅助脚本，供 GitHub Actions 和本地发布使用。

## 发布范围

Rust crates 目前仍处于 alpha 阶段。发布或作为库依赖使用之前，请先阅读 `docs/rust-oboe-release-scope.md`。当前 alpha 版本不是 C++ Oboe API 的直接替代品。

## 构建和测试

```sh
cargo fmt --manifest-path rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rust/Cargo.toml --workspace --tests -- -D warnings
cargo test --manifest-path rust/Cargo.toml
```

## Rust Alpha 发布检查

发布 Rust crates 之前先运行发布检查：

```sh
tools/check-rust-release.sh
```

如果已经安装 Rust Android targets，可以加上 Android target 检查：

```sh
CHECK_ANDROID_ABI=1 tools/check-rust-release.sh
```

第一次发布到 crates.io 时，需要先发布 `oboe-core` 并等待 registry 索引完成。`oboe-core` 可见之后，再运行依赖 crate 的 dry-run：

```sh
VERIFY_PUBLISHED_DEPS=1 tools/check-rust-release.sh
```

Android 示例应用使用的 JNI `.so` 构建属于 smoke check，不是 crates.io 发布门禁。Android 真机路径请参考 `docs/rust-android-device-smoke.md`。

## GitHub Packages

Android wrapper 可以作为 AAR 发布到 GitHub Packages：

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

发布前先构建 JNI 动态库，确保 AAR 里带有 Android `.so` 文件：

```sh
RUST_ANDROID_LIBRARIES=oboe-jni tools/build-rust-android.sh
```

本地发布到 Maven Local 做验证。release workflow 上传 GitHub Packages 之前也会先跑同样的 Maven Local 发布验证：

```sh
cd android/oboe-wrapper
./gradlew :oboe-wrapper:publishReleasePublicationToMavenLocal \
  -PoboeRust.version=0.1.0-alpha.1
```

使用有 package 写权限的 token 发布到 GitHub Packages：

```sh
cd android/oboe-wrapper
GITHUB_ACTOR=<github-user> GITHUB_TOKEN=<token> \
  ./gradlew :oboe-wrapper:publishReleasePublicationToGitHubPackagesRepository \
  -PoboeRust.version=0.1.0-alpha.1
```

仓库内置 `.github/workflows/publish-github-packages.yml`，会在 GitHub Release 发布时自动验证并发布 Android wrapper package。建议 release tag 使用 `v0.1.0-alpha.1` 这种格式；workflow 会去掉前缀 `v`，最终发布 package version `0.1.0-alpha.1`。手动触发 workflow 仍然可用，适合显式重试某个版本。

## Android Gradle Sync

用 Android Studio 打开仓库根目录。根 Gradle project 会暴露：

- `:oboe-wrapper`：面向 Java/JNI 使用方的 Android library module。
- `:oboe-smoke-app`：使用 `implementation project(':oboe-wrapper')` 的可安装 smoke app。

不通过 Gradle 编译 Java wrapper：

```powershell
javac.exe -Xlint:all -d build\javac-oboe-wrapper android\oboe-wrapper\oboe-wrapper\src\main\java\com\google\oboe\*.java
```

构建 Android JNI libraries 时需要提供 Android NDK 路径。Rust AAudio backend 会链接 `libaaudio`，所以 Android API 26 是默认 native build baseline。

```powershell
.\tools\build-rust-android.ps1 -AndroidNdk C:\path\to\Android\Sdk\ndk\<version>
```

不通过 Gradle 构建已签名 smoke-test APK：

```powershell
.\tools\build-smoke-apk.ps1 -AndroidSdk C:\path\to\Android\Sdk
```

安装到已连接的真机或模拟器：

```powershell
.\tools\build-smoke-apk.ps1 -AndroidSdk C:\path\to\Android\Sdk -Install
```
