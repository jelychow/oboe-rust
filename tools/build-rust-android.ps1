param(
    [Parameter(Mandatory = $true)]
    [string]$AndroidNdk,

    [string]$ApiLevel = "26"
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$manifestPath = Join-Path $repoRoot "rust/Cargo.toml"
$outRoot = Join-Path $repoRoot "android/oboe-wrapper/oboe-wrapper/src/main/jniLibs"
$toolchainBin = Join-Path $AndroidNdk "toolchains/llvm/prebuilt/windows-x86_64/bin"

if (-not (Test-Path -LiteralPath $manifestPath)) {
    throw "Rust manifest not found at '$manifestPath'."
}

if (-not (Test-Path -LiteralPath $toolchainBin)) {
    throw "Android NDK LLVM toolchain bin directory not found at '$toolchainBin'."
}

$targets = @(
    @{
        Abi = "arm64-v8a"
        Triple = "aarch64-linux-android"
        Linker = "aarch64-linux-android$ApiLevel-clang.cmd"
        LinkerEnv = "CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER"
    },
    @{
        Abi = "armeabi-v7a"
        Triple = "armv7-linux-androideabi"
        Linker = "armv7a-linux-androideabi$ApiLevel-clang.cmd"
        LinkerEnv = "CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER"
    },
    @{
        Abi = "x86"
        Triple = "i686-linux-android"
        Linker = "i686-linux-android$ApiLevel-clang.cmd"
        LinkerEnv = "CARGO_TARGET_I686_LINUX_ANDROID_LINKER"
    },
    @{
        Abi = "x86_64"
        Triple = "x86_64-linux-android"
        Linker = "x86_64-linux-android$ApiLevel-clang.cmd"
        LinkerEnv = "CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER"
    }
)

foreach ($target in $targets) {
    $linkerPath = Join-Path $toolchainBin $target.Linker
    if (-not (Test-Path -LiteralPath $linkerPath)) {
        throw "Missing Android NDK linker '$($target.Linker)' for target '$($target.Triple)'. Checked '$linkerPath'."
    }

    [Environment]::SetEnvironmentVariable($target.LinkerEnv, $linkerPath, "Process")

    & cargo build --manifest-path $manifestPath -p oboe-jni --release --target $target.Triple
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed for target '$($target.Triple)' with exit code $LASTEXITCODE."
    }

    $builtLibrary = Join-Path $repoRoot "rust/target/$($target.Triple)/release/liboboe_jni.so"
    if (-not (Test-Path -LiteralPath $builtLibrary)) {
        throw "Expected Rust library not found at '$builtLibrary'."
    }

    $abiOutputDir = Join-Path $outRoot $target.Abi
    New-Item -ItemType Directory -Force -Path $abiOutputDir | Out-Null
    Copy-Item -LiteralPath $builtLibrary -Destination (Join-Path $abiOutputDir "liboboe_jni.so") -Force
}
