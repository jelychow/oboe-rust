#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
android_ndk_version="${ANDROID_NDK_VERSION:-29.0.14206865}"
android_compile_sdk="${ANDROID_COMPILE_SDK:-34}"
android_build_tools_version="${ANDROID_BUILD_TOOLS_VERSION:-34.0.0}"

if [[ -z "${ANDROID_HOME:-}" ]]; then
  export ANDROID_HOME="${ANDROID_SDK_ROOT:-$repo_root/.local/android-sdk}"
fi
export ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-$ANDROID_HOME}"

find_sdkmanager() {
  local sdkmanager_path="$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager"

  if [[ -x "$sdkmanager_path" ]]; then
    printf '%s\n' "$sdkmanager_path"
    return 0
  fi

  command -v sdkmanager || true
}

load_cargo_env() {
  if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck disable=SC1090
    . "$HOME/.cargo/env"
  fi
}

ensure_rust_toolchain() {
  load_cargo_env

  if ! command -v cargo >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --profile minimal --default-toolchain stable
    load_cargo_env
  fi

  rustup target add \
    aarch64-linux-android \
    armv7-linux-androideabi \
    i686-linux-android \
    x86_64-linux-android
}

ensure_android_sdk() {
  if [[ -z "${ANDROID_NDK:-}" ]]; then
    export ANDROID_NDK="$ANDROID_HOME/ndk/$android_ndk_version"
  fi

  if [[ -d "$ANDROID_NDK/toolchains/llvm/prebuilt" \
    && -d "$ANDROID_HOME/platforms/android-$android_compile_sdk" \
    && -d "$ANDROID_HOME/build-tools/$android_build_tools_version" ]]; then
    return 0
  fi

  local sdkmanager_path
  sdkmanager_path="$(find_sdkmanager)"
  if [[ -z "$sdkmanager_path" ]]; then
    echo "sdkmanager was not found. Set ANDROID_NDK or install Android command-line tools." >&2
    exit 1
  fi

  (yes || true) | "$sdkmanager_path" --licenses >/dev/null || true
  "$sdkmanager_path" \
    "platforms;android-$android_compile_sdk" \
    "build-tools;$android_build_tools_version" \
    "platform-tools" \
    "ndk;$android_ndk_version"
}

jitpack_group_id() {
  if [[ -n "${JITPACK_GROUP_ID:-}" ]]; then
    printf '%s\n' "$JITPACK_GROUP_ID"
  elif [[ -n "${GROUP:-}" && -n "${ARTIFACT:-}" ]]; then
    printf '%s.%s\n' "$GROUP" "$ARTIFACT"
  else
    printf '%s\n' "com.github.jelychow.oboe-rust"
  fi
}

jitpack_artifact_id() {
  printf '%s\n' "${JITPACK_ARTIFACT_ID:-oboe-wrapper}"
}

jitpack_version() {
  if [[ -n "${JITPACK_VERSION:-}" ]]; then
    printf '%s\n' "$JITPACK_VERSION"
  elif [[ -n "${VERSION:-}" ]]; then
    printf '%s\n' "$VERSION"
  else
    printf '%s\n' "0.1.0-alpha.6"
  fi
}

ensure_rust_toolchain
ensure_android_sdk

export RUST_ANDROID_LIBRARIES="${RUST_ANDROID_LIBRARIES:-oboe-jni}"
"$repo_root/tools/build-rust-android.sh"

group_id="$(jitpack_group_id)"
artifact_id="$(jitpack_artifact_id)"
version="$(jitpack_version)"

"$repo_root/gradlew" \
  :oboe-wrapper:publishReleasePublicationToMavenLocal \
  -PoboeRust.groupId="$group_id" \
  -PoboeRust.artifactId="$artifact_id" \
  -PoboeRust.version="$version" \
  -PoboeRust.githubOwner="${JITPACK_GITHUB_OWNER:-jelychow}" \
  -PoboeRust.githubRepository="${JITPACK_GITHUB_REPOSITORY:-oboe-rust}" \
  --no-daemon \
  --console=plain

echo "Published JitPack artifact to Maven Local: $group_id:$artifact_id:$version"
