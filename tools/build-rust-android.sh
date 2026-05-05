#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
main_manifest_path="$repo_root/rust/Cargo.toml"
examples_manifest_path="$repo_root/examples/rust/Cargo.toml"
api_level="${API_LEVEL:-26}"
android_ndk="${ANDROID_NDK:-$repo_root/.local/android-sdk/ndk/29.0.14206865}"
toolchain_root="$android_ndk/toolchains/llvm/prebuilt"
target_dir="${CARGO_TARGET_DIR:-$repo_root/rust/target}"
selected_libraries="${RUST_ANDROID_LIBRARIES:-}"

if [[ -n "${ANDROID_NDK_HOST_TAG:-}" ]]; then
  host_tag="$ANDROID_NDK_HOST_TAG"
else
  case "$(uname -s)" in
    Linux*) host_tag="linux-x86_64" ;;
    Darwin*)
      if [[ -d "$toolchain_root/darwin-aarch64" ]]; then
        host_tag="darwin-aarch64"
      else
        host_tag="darwin-x86_64"
      fi
      ;;
    MINGW* | MSYS* | CYGWIN*) host_tag="windows-x86_64" ;;
    *)
      echo "Unsupported host OS '$(uname -s)'. Set ANDROID_NDK_HOST_TAG to override." >&2
      exit 1
      ;;
  esac
fi

toolchain_bin="$toolchain_root/$host_tag/bin"

if [[ ! -f "$main_manifest_path" ]]; then
  echo "Rust manifest not found at '$main_manifest_path'." >&2
  exit 1
fi

if [[ ! -f "$examples_manifest_path" ]]; then
  echo "Examples Rust manifest not found at '$examples_manifest_path'." >&2
  exit 1
fi

if [[ ! -d "$toolchain_bin" ]]; then
  echo "Android NDK toolchain bin directory not found at '$toolchain_bin'." >&2
  exit 1
fi

resolve_tool() {
  local tool="$1"
  if [[ -x "$toolchain_bin/$tool" ]]; then
    printf '%s\n' "$toolchain_bin/$tool"
  elif [[ -x "$toolchain_bin/$tool.cmd" ]]; then
    printf '%s\n' "$toolchain_bin/$tool.cmd"
  elif [[ -x "$toolchain_bin/$tool.exe" ]]; then
    printf '%s\n' "$toolchain_bin/$tool.exe"
  else
    echo "Missing Android NDK tool '$tool' in '$toolchain_bin'." >&2
    return 1
  fi
}

targets=(
  "arm64-v8a|aarch64-linux-android|aarch64-linux-android${api_level}-clang|CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER|aarch64_linux_android"
  "armeabi-v7a|armv7-linux-androideabi|armv7a-linux-androideabi${api_level}-clang|CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER|armv7_linux_androideabi"
  "x86|i686-linux-android|i686-linux-android${api_level}-clang|CARGO_TARGET_I686_LINUX_ANDROID_LINKER|i686_linux_android"
  "x86_64|x86_64-linux-android|x86_64-linux-android${api_level}-clang|CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER|x86_64_linux_android"
)

libraries=(
  "oboe-jni|$main_manifest_path|liboboe_jni.so|$repo_root/android/oboe-wrapper/oboe-wrapper/src/main/jniLibs"
  "oboe-samples-jni|$examples_manifest_path|liboboe_samples_jni.so|$repo_root/android/oboe-wrapper/oboe-samples-app/src/main/jniLibs"
)

trim_space() {
  local value="$1"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s' "$value"
}

should_build_library() {
  local package="$1"
  local selected

  if [[ -z "$selected_libraries" ]]; then
    return 0
  fi

  IFS=',' read -ra selected <<< "$selected_libraries"
  for selected_package in "${selected[@]}"; do
    selected_package="$(trim_space "$selected_package")"
    if [[ "$selected_package" == "$package" ]]; then
      return 0
    fi
  done

  return 1
}

validate_selected_libraries() {
  local selected
  local selected_package
  local library
  local package
  local package_manifest
  local library_name
  local out_root
  local found

  if [[ -z "$selected_libraries" ]]; then
    return 0
  fi

  IFS=',' read -ra selected <<< "$selected_libraries"
  for selected_package in "${selected[@]}"; do
    selected_package="$(trim_space "$selected_package")"
    found=0

    for library in "${libraries[@]}"; do
      IFS='|' read -r package package_manifest library_name out_root <<< "$library"
      if [[ "$selected_package" == "$package" ]]; then
        found=1
        break
      fi
    done

    if [[ "$found" -ne 1 ]]; then
      echo "Unknown Rust Android library '$selected_package' in RUST_ANDROID_LIBRARIES." >&2
      exit 1
    fi
  done
}

validate_selected_libraries

for target in "${targets[@]}"; do
  IFS='|' read -r abi triple linker linker_env cc_env_suffix <<< "$target"
  linker_path="$(resolve_tool "$linker")"
  ar_path="$(resolve_tool llvm-ar)"

  export "$linker_env=$linker_path"
  export "CC_$cc_env_suffix=$linker_path"
  export "AR_$cc_env_suffix=$ar_path"

  for library in "${libraries[@]}"; do
    IFS='|' read -r package package_manifest library_name out_root <<< "$library"
    if ! should_build_library "$package"; then
      continue
    fi

    cargo build \
      --manifest-path "$package_manifest" \
      -p "$package" \
      --release \
      --target "$triple" \
      --target-dir "$target_dir"

    built_library="$target_dir/$triple/release/$library_name"
    if [[ ! -f "$built_library" ]]; then
      echo "Expected Rust library not found at '$built_library'." >&2
      exit 1
    fi

    abi_output_dir="$out_root/$abi"
    mkdir -p "$abi_output_dir"
    cp "$built_library" "$abi_output_dir/$library_name"
  done
done
