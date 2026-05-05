# OpenAI Realtime App

The app can be built against either the local SDK source module or the
published GitHub Packages AAR.

Default local SDK source build:

```bash
RUST_ANDROID_LIBRARIES=oboe-jni tools/build-rust-android.sh
cd android/oboe-wrapper
./gradlew :openai-realtime-app:assembleDebug --no-daemon --console=plain
```

Rerun the Rust build step whenever `rust/oboe-jni` or the Java wrapper native
method surface changes. The app checks the native library version before
opening streams and reports a version mismatch if `liboboe_jni.so` is stale.

Remote AAR build:

```properties
# local.properties in the repository root or android/oboe-wrapper/
gpr.key=<token-with-read:packages>
gpr.user=<github-user>
oboeRust.realtimeDependency=remote
oboeRust.remoteVersion=0.1.0-alpha.6
```

```bash
./gradlew :openai-realtime-app:assembleDebug --no-daemon --console=plain
```

The dependency source can also be passed on the command line:

```bash
./gradlew :openai-realtime-app:assembleDebug -PoboeRust.realtimeDependency=source --no-daemon --console=plain
./gradlew :openai-realtime-app:assembleDebug -PoboeRust.realtimeDependency=remote -PoboeRust.remoteVersion=0.1.0-alpha.6 --no-daemon --console=plain
```

The PowerShell helper forwards the same switch:

```powershell
.\tools\build-openai-realtime-apk.ps1 -OboeDependency source
.\tools\build-openai-realtime-apk.ps1 -OboeDependency remote -OboeRemoteVersion 0.1.0-alpha.6
```

Accepted source values are `source`, `sdk`, `local`, `remote`, and `aar`.

The app's own `com.example.openairustrealtime` classes are pinned into the
primary dex so Android can instantiate the launcher activity before any
secondary dex loading edge cases.
