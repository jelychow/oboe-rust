# oboe-rust

[English](README.md)

这个仓库现在同时维护 **两条 Android 接入路径**：

- **Route C / C++ 消费路径**：恢复了面向 Android 游戏项目的 C++ 公共头文件、根 CMake 入口和 Prefab 元数据，适合依赖 C++ headers、回调驱动实时音频、稳定 buffer 控制、xrun/underrun 可观测性、route/device change 处理以及低延迟设备适配能力的工程。
- **Rust-native 路径**：保留 Rust crates、JNI bridge 和 Android Java wrapper，继续服务 Rust-first 的实验和发布流程。

当前仓库内的主要组成包括：

- `include/oboe`：恢复的 C++ 公共头文件，供原生 Android / 游戏引擎消费者使用。
- `CMakeLists.txt`：C++ / Prefab 路径的根 CMake 入口。
- `prefab`：恢复的 Android 打包元数据骨架。
- `rust/oboe-core`：与后端无关的 stream、builder、FIFO、format、resampler、callback 和扩展状态。
- `rust/oboe-android`：Android AAudio 和 OpenSL ES 后端 FFI。
- `rust/oboe-jni`：暴露给 Java 的 JNI handle 层。
- `android/oboe-wrapper`：Android Java wrapper 和 smoke test。
- `examples/rust`：用于 Android demo 的 JNI crates，负责连接 wrapper 和示例 app。
- `tools/build-rust-android.ps1`：Windows 上构建 Android ABI `liboboe_jni.so` 的辅助脚本。
- `tools/build-rust-android.sh`：Linux/macOS 上构建 Android ABI 的辅助脚本，供 GitHub Actions 和本地发布使用。

Demo 专用 native bridge 仍然不放在可发布 Rust workspace 里。
`examples/rust/oboe-samples-jni` 支撑样例 launcher。保留的
`android/oboe-wrapper/openai-realtime-app` demo 使用 Kotlin/Ktor 处理
OpenAI Realtime 网络，并通过 Android SDK wrapper 使用 Oboe。

## 发布范围

Rust crates 目前仍处于 alpha 阶段。发布或作为库依赖使用之前，请先阅读 `docs/rust-oboe-release-scope.md`。Rust alpha 版本**不是** C++ Oboe API 的直接替代品。对于依赖 C++ headers / CMake / Prefab、需要 callback 驱动实时 I/O 的 Android 游戏项目，应优先使用恢复的 Route C 路径，并在真机上完成验证。

## 构建和测试

```sh
tools/check-public-surface.sh
cargo fmt --manifest-path rust/Cargo.toml --all -- --check
cargo fmt --manifest-path examples/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rust/Cargo.toml --workspace --tests -- -D warnings
cargo clippy --manifest-path examples/rust/Cargo.toml --workspace --tests -- -D warnings
cargo test --manifest-path rust/Cargo.toml
cargo test --manifest-path examples/rust/Cargo.toml
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

GitHub Packages 安装 package 时也需要认证。Gradle 本地使用时可以在 `~/.gradle/gradle.properties` 里配置 `gpr.user` 和 `gpr.key`，CI 里可以使用 `GITHUB_ACTOR` 和 `GITHUB_TOKEN`：

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

如果使用 Apache Maven，需要在 `~/.m2/settings.xml` 配置 GitHub Packages repository 和 server 凭证，然后 dependency 里声明这是 AAR：

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
发布完成后，workflow 还会临时创建一个 Android app，从 GitHub Packages 远端拉取刚发布的 package 编译，并检查 APK 里是否包含四个 ABI 的 `liboboe_jni.so`。

## JitPack

JitPack 可以从公开 GitHub 仓库构建 Android wrapper，不需要 GitHub Packages 凭证。先添加 JitPack repository：

JitPack public 下载要求这个 GitHub 仓库本身是公开可见的。如果 JitPack 返回 `Repo not found or no token provided`，需要把仓库改成 public，或者配置 JitPack 的私有仓库访问。

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

然后依赖 wrapper module：

```groovy
dependencies {
    implementation("com.github.jelychow.oboe-rust:oboe-wrapper:<tag-or-commit>")
}
```

例如可以先用 `main-SNAPSHOT` 测试最新 `main` 分支构建；正式使用时，在这次 JitPack 配置合并后创建一个新的 Git tag，然后把 tag 当作 version。旧的上游 tag 早于当前 Rust/JitPack 发布脚本，不适合用来拉这个 package。

根目录 `jitpack.yml` 会运行 `tools/publish-jitpack-android.sh`。这个脚本会按需安装 Rust targets 和 Android NDK，只构建 `liboboe_jni.so`，然后使用 JitPack 多模块坐标发布 Android wrapper 到 Maven Local：

```sh
JITPACK_GROUP_ID=com.github.jelychow.oboe-rust \
JITPACK_ARTIFACT_ID=oboe-wrapper \
JITPACK_VERSION=main-SNAPSHOT \
  tools/publish-jitpack-android.sh
```

## Android Gradle Sync

用 Android Studio 打开仓库根目录。根 Gradle project 会暴露：

- `:oboe-wrapper`：面向 Java/JNI 使用方的 Android library module。
- `:oboe-smoke-app`：使用 `implementation project(':oboe-wrapper')` 的可安装 smoke app。
- `:oboe-samples-app`：运行 Rust sample engines 的 Android launcher。
- `:openai-realtime-app`：保留的 OpenAI realtime/TTS/ASR demo，Realtime 网络使用 Ktor，音频路径使用 Oboe SDK API。

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
