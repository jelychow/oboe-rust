#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

require_file() {
  local path="$1"
  if [[ ! -f "$repo_root/$path" ]]; then
    echo "Expected file '$path' to exist for live Route C build support." >&2
    exit 1
  fi
}

require_dir() {
  local path="$1"
  if [[ ! -d "$repo_root/$path" ]]; then
    echo "Expected directory '$path' to exist for live Route C build support." >&2
    exit 1
  fi
}

require_dir "src"
require_dir "tests"
require_dir "rust/oboe_rust_core"
require_file "prefab_build.sh"
require_file "include/oboe/AudioStreamCallback.h"

if ! grep -q 'option(OBOE_BUILD_LEGACY_CPP .* ON)' "$repo_root/CMakeLists.txt"; then
  echo "Expected OBOE_BUILD_LEGACY_CPP to default to ON for Route C consumers." >&2
  exit 1
fi

echo "Route C live build surface is present."
