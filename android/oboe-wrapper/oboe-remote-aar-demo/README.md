# Oboe Remote AAR Demo

This app uses the published GitHub Packages artifact instead of the local
`:oboe-wrapper` project module.

Dependency:

```gradle
implementation "io.github.jelychow.oboe:oboe-rust-android:0.1.0-alpha.6"
```

Local builds read GitHub Packages credentials from `local.properties`:

```properties
gpr.key=<token-with-read:packages>
# Optional; defaults to jelychow when gpr.key is present.
gpr.user=<github-user>
```

The root build reads `local.properties` from the repository root or
`android/oboe-wrapper/local.properties`. The standalone demo also reads the
same parent `local.properties` files.

Then build from the repository root:

```bash
./gradlew :oboe-remote-aar-demo:assembleDebug --no-daemon --console=plain
```

It can also be opened or built as a standalone Gradle project:

```bash
cd android/oboe-wrapper/oboe-remote-aar-demo
../gradlew assembleDebug --no-daemon --console=plain
```

The version can be overridden with `-PoboeRust.remoteVersion=<version>`.
