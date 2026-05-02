#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

require_file() {
  local path="$1"
  if [[ ! -f "$repo_root/$path" ]]; then
    echo "Expected file '$path' to exist." >&2
    exit 1
  fi
}

require_dir() {
  local path="$1"
  if [[ ! -d "$repo_root/$path" ]]; then
    echo "Expected directory '$path' to exist." >&2
    exit 1
  fi
}

require_absent() {
  local path="$1"
  if [[ -e "$repo_root/$path" ]]; then
    echo "Expected '$path' to be removed from the public surface." >&2
    exit 1
  fi
}

reject_pattern() {
  local file="$1"
  local pattern="$2"
  if grep -qE "$pattern" "$repo_root/$file"; then
    echo "Unexpected pattern '$pattern' in '$file'." >&2
    exit 1
  fi
}

require_file "rust/Cargo.toml"
require_file "examples/rust/Cargo.toml"
require_file "android/oboe-wrapper/settings.gradle"
require_file "settings.gradle"
require_file "tools/build-rust-android.sh"
require_file "tools/build-rust-android.ps1"
require_dir "android/oboe-wrapper/openai-realtime-app"
require_dir "examples/rust/oboe-samples-jni"

require_absent "rust/openai-realtime-jni"
require_absent "examples/rust/openai-realtime-jni"
require_absent "rust/oboe-samples-jni"
require_absent "rust/minimaloboe-rust-jni"
require_absent "android/oboe-wrapper/minimaloboe-rust-app"
require_absent "tools/build-minimaloboe-rust-apk.ps1"
require_absent "android/oboe-wrapper/openai-realtime-app/src/main/jniLibs"

reject_pattern "rust/Cargo.toml" 'openai-realtime-jni|oboe-samples-jni|minimaloboe-rust-jni'
reject_pattern "examples/rust/Cargo.toml" 'openai-realtime-jni'
reject_pattern "android/oboe-wrapper/settings.gradle" 'minimaloboe-rust-app'
reject_pattern "settings.gradle" 'minimaloboe-rust-app'
reject_pattern "tools/build-rust-android.sh" 'minimaloboe-rust-jni|minimaloboe-rust-app|openai-realtime-jni|libopenai_realtime_jni'
reject_pattern "tools/build-rust-android.ps1" 'minimaloboe-rust-jni|minimaloboe-rust-app|openai-realtime-jni|libopenai_realtime_jni'

echo "Public surface structure is clean."
