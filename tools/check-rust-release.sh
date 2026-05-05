#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
manifest="$repo_root/rust/Cargo.toml"

release_crates=(
  oboe-core
  oboe-android
  oboe-samples
)

release_package_args=()
for crate in "${release_crates[@]}"; do
  release_package_args+=("-p" "$crate")
done

cargo fmt --manifest-path "$manifest" "${release_package_args[@]}" --check
cargo clippy --manifest-path "$manifest" "${release_package_args[@]}" --tests -- -D warnings
cargo test --manifest-path "$manifest" "${release_package_args[@]}"

cargo publish --dry-run --manifest-path "$manifest" -p oboe-core --allow-dirty

dependent_crates=(
  oboe-android
  oboe-samples
)

if [[ "${VERIFY_PUBLISHED_DEPS:-0}" == "1" ]]; then
  for crate in "${dependent_crates[@]}"; do
    cargo publish --dry-run --manifest-path "$manifest" -p "$crate" --allow-dirty
  done
else
  for crate in "${dependent_crates[@]}"; do
    cargo package --manifest-path "$manifest" -p "$crate" --allow-dirty --list >/dev/null
  done
  echo "Skipped publish dry-run for oboe-android and oboe-samples because oboe-core must exist in the registry first."
  echo "After oboe-core is published and indexed, rerun with VERIFY_PUBLISHED_DEPS=1."
fi

if [[ "${CHECK_ANDROID_ABI:-0}" == "1" ]]; then
  android_targets=(
    aarch64-linux-android
    armv7-linux-androideabi
    i686-linux-android
    x86_64-linux-android
  )

  for target in "${android_targets[@]}"; do
    cargo check --manifest-path "$manifest" "${release_package_args[@]}" --target "$target"
  done
  echo "Checked publishable Rust crates for Android ABI targets."
else
  echo "Skipping Android target check. Set CHECK_ANDROID_ABI=1 to check publishable crates for Android targets."
fi
