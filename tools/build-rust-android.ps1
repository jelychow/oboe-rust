param(
    [Parameter(Mandatory = $true)]
    [string]$AndroidNdk,

    [string]$ApiLevel = "26"
)

$ErrorActionPreference = "Stop"

if ($env:PATHEXT -notmatch '(^|;)\.EXE(;|$)') {
    $env:PATHEXT = ".COM;.EXE;.BAT;.CMD;$env:PATHEXT"
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$manifestPath = Join-Path $repoRoot "rust/Cargo.toml"
$wrapperOutRoot = Join-Path $repoRoot "android/oboe-wrapper/oboe-wrapper/src/main/jniLibs"
$samplesOutRoot = Join-Path $repoRoot "android/oboe-wrapper/oboe-samples-app/src/main/jniLibs"
$minimalOboeOutRoot = Join-Path $repoRoot "android/oboe-wrapper/minimaloboe-rust-app/src/main/jniLibs"
$openAiRealtimeOutRoot = Join-Path $repoRoot "android/oboe-wrapper/openai-realtime-app/src/main/jniLibs"
$toolchainBin = Join-Path $AndroidNdk "toolchains/llvm/prebuilt/windows-x86_64/bin"

if (-not (Test-Path -LiteralPath $manifestPath)) {
    throw "Rust manifest not found at '$manifestPath'."
}

if (-not (Test-Path -LiteralPath $toolchainBin)) {
    throw "Android NDK LLVM toolchain bin directory not found at '$toolchainBin'."
}

$cargoCommand = Get-Command cargo.exe -ErrorAction SilentlyContinue
if ($null -eq $cargoCommand) {
    $cargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
}
if ($null -eq $cargoCommand) {
    throw "cargo executable not found on PATH."
}

function Invoke-ExternalCommand($filePath, $arguments, $failureMessage) {
    $argumentLine = ($arguments | ForEach-Object {
        $argument = [string]$_
        if ($argument -match '[\s"]') {
            '"' + ($argument -replace '"', '\"') + '"'
        } else {
            $argument
        }
    }) -join " "

    $process = Start-Process `
        -FilePath $filePath `
        -ArgumentList $argumentLine `
        -Wait `
        -PassThru `
        -NoNewWindow
    if ($process.ExitCode -ne 0) {
        throw "$failureMessage Exit code $($process.ExitCode)."
    }
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

$libraryBuilds = @(
    @{
        Package = "oboe-jni"
        Library = "liboboe_jni.so"
        OutRoot = $wrapperOutRoot
    },
    @{
        Package = "oboe-samples-jni"
        Library = "liboboe_samples_jni.so"
        OutRoot = $samplesOutRoot
    },
    @{
        Package = "minimaloboe-rust-jni"
        Library = "libminimaloboe_rust.so"
        OutRoot = $minimalOboeOutRoot
    },
    @{
        Package = "openai-realtime-jni"
        Library = "libopenai_realtime_jni.so"
        OutRoot = $openAiRealtimeOutRoot
    }
)

foreach ($target in $targets) {
    $linkerPath = Join-Path $toolchainBin $target.Linker
    if (-not (Test-Path -LiteralPath $linkerPath)) {
        throw "Missing Android NDK linker '$($target.Linker)' for target '$($target.Triple)'. Checked '$linkerPath'."
    }

    [Environment]::SetEnvironmentVariable($target.LinkerEnv, $linkerPath, "Process")

    foreach ($libraryBuild in $libraryBuilds) {
        Invoke-ExternalCommand `
            $cargoCommand.Source `
            @("build", "--manifest-path", $manifestPath, "-p", $libraryBuild.Package, "--release", "--target", $target.Triple) `
            "cargo build failed for package '$($libraryBuild.Package)' target '$($target.Triple)'."

        $builtLibrary = Join-Path $repoRoot "rust/target/$($target.Triple)/release/$($libraryBuild.Library)"
        if (-not (Test-Path -LiteralPath $builtLibrary)) {
            throw "Expected Rust library not found at '$builtLibrary'."
        }

        $abiOutputDir = Join-Path $libraryBuild.OutRoot $target.Abi
        New-Item -ItemType Directory -Force -Path $abiOutputDir | Out-Null
        Copy-Item -LiteralPath $builtLibrary -Destination (Join-Path $abiOutputDir $libraryBuild.Library) -Force
    }
}
