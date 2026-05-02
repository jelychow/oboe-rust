#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

require_file() {
  local path="$1"
  if [[ ! -f "$repo_root/$path" ]]; then
    echo "Expected file '$path' to exist for realtime Route C support." >&2
    exit 1
  fi
}

require_dir() {
  local path="$1"
  if [[ ! -d "$repo_root/$path" ]]; then
    echo "Expected directory '$path' to exist for realtime Route C support." >&2
    exit 1
  fi
}

require_file "include/oboe/AudioStreamCallback.h"
require_file "include/oboe/LatencyTuner.h"
require_file "src/common/LatencyTuner.cpp"
require_file "src/common/StabilizedCallback.cpp"
require_file "src/aaudio/AudioStreamAAudio.cpp"
require_file "src/opensles/AudioStreamOpenSLES.cpp"
require_dir "apps/OboeTester"
require_file "apps/OboeTester/app/src/main/cpp/TestErrorCallback.cpp"
require_file "apps/OboeTester/app/src/main/cpp/TestRoutingCrash.cpp"
require_file "tests/testXRunBehaviour.cpp"

echo "Route C realtime capability surface is present."
