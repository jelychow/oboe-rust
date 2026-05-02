#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

require_file() {
  local path="$1"
  if [[ ! -f "$repo_root/$path" ]]; then
    echo "Expected file '$path' to exist for Route C foundation." >&2
    exit 1
  fi
}

require_dir() {
  local path="$1"
  if [[ ! -d "$repo_root/$path" ]]; then
    echo "Expected directory '$path' to exist for Route C foundation." >&2
    exit 1
  fi
}

require_file "CMakeLists.txt"
require_dir "include/oboe"
require_file "include/oboe/Oboe.h"
require_file "include/oboe/AudioStreamBuilder.h"
require_file "include/oboe/AudioStreamCallback.h"
require_dir "prefab"
require_file "prefab/oboe-VERSION/prefab/prefab.json"
require_file "prefab/oboe-VERSION/prefab/modules/oboe/module.json"
require_file "rust/Cargo.toml"
require_dir "android/oboe-wrapper"

echo "Route C foundation surface is present."
