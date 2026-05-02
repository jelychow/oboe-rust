param(
    [string]$AndroidSdk = $(if ($env:ANDROID_HOME) { $env:ANDROID_HOME } else { "F:\Android\android-sdk" }),
    [string]$MinSdk = "26",
    [string]$TargetSdk = "34",
    [string]$BuildToolsVersion = "",
    [string]$NdkVersion = "",
    [string]$OutputApk = "",
    [switch]$SkipRustBuild,
    [switch]$Install
)

$ErrorActionPreference = "Stop"

if ($env:PATHEXT -notmatch '(^|;)\.EXE(;|$)') {
    $env:PATHEXT = ".COM;.EXE;.BAT;.CMD;$env:PATHEXT"
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$appJavaDir = Join-Path $repoRoot "android/oboe-wrapper/openai-realtime-app/src/main/java"
$appKotlinDir = Join-Path $repoRoot "android/oboe-wrapper/openai-realtime-app/src/main/kotlin"
$appManifest = Join-Path $repoRoot "android/oboe-wrapper/openai-realtime-app/src/main/AndroidManifest.xml"
$gradleProjectDir = Join-Path $repoRoot "android/oboe-wrapper"
$gradleWrapper = Join-Path $gradleProjectDir "gradlew.bat"
$buildRoot = Join-Path $repoRoot "build/openai-realtime-apk"
$classesDir = Join-Path $buildRoot "classes"
$dexDir = Join-Path $buildRoot "dex"
$packagingDir = Join-Path $buildRoot "package"
$unsignedApk = Join-Path $buildRoot "openai-realtime-unsigned.apk"
$alignedApk = Join-Path $buildRoot "openai-realtime-aligned.apk"
$androidUserHome = $(if ($env:ANDROID_USER_HOME) { $env:ANDROID_USER_HOME } else { Join-Path $env:USERPROFILE ".android" })
$keystore = Join-Path $androidUserHome "debug.keystore"

if ([string]::IsNullOrWhiteSpace($OutputApk)) {
    $OutputApk = Join-Path $buildRoot "openai-realtime-debug.apk"
}

function Get-LatestDirectory($path) {
    $directory = Get-ChildItem -LiteralPath $path -Directory | Sort-Object Name | Select-Object -Last 1
    if ($null -eq $directory) {
        throw "No directory entries found under '$path'."
    }
    $directory.FullName
}

if (-not (Test-Path -LiteralPath $AndroidSdk)) {
    throw "Android SDK not found at '$AndroidSdk'."
}

if ([string]::IsNullOrWhiteSpace($BuildToolsVersion)) {
    $buildToolsDir = Get-LatestDirectory (Join-Path $AndroidSdk "build-tools")
} else {
    $buildToolsDir = Join-Path $AndroidSdk "build-tools/$BuildToolsVersion"
}

if ([string]::IsNullOrWhiteSpace($NdkVersion)) {
    $ndkDir = Get-LatestDirectory (Join-Path $AndroidSdk "ndk")
} else {
    $ndkDir = Join-Path $AndroidSdk "ndk/$NdkVersion"
}

$platformDir = Join-Path $AndroidSdk "platforms/android-$TargetSdk"
$androidJar = Join-Path $platformDir "android.jar"
$aapt2 = Join-Path $buildToolsDir "aapt2.exe"
$d8 = Join-Path $buildToolsDir "d8.bat"
$zipalign = Join-Path $buildToolsDir "zipalign.exe"
$apksigner = Join-Path $buildToolsDir "apksigner.bat"
$adb = Join-Path $AndroidSdk "platform-tools/adb.exe"
$javac = (Get-Command javac.exe -ErrorAction Stop).Source
$keytool = (Get-Command keytool.exe -ErrorAction Stop).Source

foreach ($path in @($androidJar, $aapt2, $d8, $zipalign, $apksigner, $adb, $javac, $keytool, $appManifest)) {
    if (-not (Test-Path -LiteralPath $path)) {
        throw "Required Android tool or input is missing: '$path'."
    }
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

$cargoBin = Join-Path $env:USERPROFILE ".cargo/bin"
if (Test-Path -LiteralPath (Join-Path $cargoBin "cargo.exe")) {
    $env:PATH = "$cargoBin;$env:PATH"
}

if (-not $SkipRustBuild) {
    & (Join-Path $PSScriptRoot "build-rust-android.ps1") -AndroidNdk $ndkDir -ApiLevel $MinSdk
}

$hasKotlinSources = (Test-Path -LiteralPath $appKotlinDir) -and
    $null -ne (Get-ChildItem -LiteralPath $appKotlinDir -Filter *.kt -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1)
if ($hasKotlinSources) {
    if (-not (Test-Path -LiteralPath $gradleWrapper)) {
        throw "Gradle wrapper not found at '$gradleWrapper'. Kotlin sources require the Gradle Android build path."
    }

    Invoke-ExternalCommand `
        $gradleWrapper `
        @("--no-daemon", ":openai-realtime-app:assembleDebug", "--console=plain") `
        "Gradle failed for OpenAI Realtime Kotlin APK sources."

    $gradleApk = Join-Path $gradleProjectDir "openai-realtime-app/build/outputs/apk/debug/openai-realtime-app-debug.apk"
    if (-not (Test-Path -LiteralPath $gradleApk)) {
        throw "Gradle did not produce expected APK: '$gradleApk'."
    }

    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $OutputApk) | Out-Null
    Copy-Item -LiteralPath $gradleApk -Destination $OutputApk -Force
    Write-Output "OpenAI Realtime APK built: $OutputApk"

    if ($Install) {
        Invoke-ExternalCommand `
            $adb `
            @("install", "-r", $OutputApk) `
            "adb install failed."
    }
    return
}

Remove-Item -Recurse -Force -LiteralPath $buildRoot -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $classesDir, $dexDir, $packagingDir | Out-Null

$manualManifest = Join-Path $buildRoot "AndroidManifest.xml"
$manifestText = Get-Content -Raw -LiteralPath $appManifest
$manifestText = $manifestText -replace '<manifest xmlns:android="http://schemas.android.com/apk/res/android">', '<manifest xmlns:android="http://schemas.android.com/apk/res/android" package="com.example.openairustrealtime">'
Set-Content -LiteralPath $manualManifest -Value $manifestText -Encoding UTF8

$javaSources = Get-ChildItem -LiteralPath $appJavaDir -Filter *.java -Recurse | ForEach-Object { $_.FullName }

Invoke-ExternalCommand `
    $javac `
    (@("-source", "8", "-target", "8", "-encoding", "UTF-8", "-bootclasspath", $androidJar, "-d", $classesDir) + $javaSources) `
    "javac failed for OpenAI Realtime APK sources."

$classFiles = Get-ChildItem -LiteralPath $classesDir -Filter *.class -Recurse | ForEach-Object { $_.FullName }
Invoke-ExternalCommand `
    $d8 `
    (@("--min-api", $MinSdk, "--lib", $androidJar, "--output", $dexDir) + $classFiles) `
    "d8 failed for OpenAI Realtime APK classes."

Invoke-ExternalCommand `
    $aapt2 `
    @("link", "-o", $unsignedApk, "-I", $androidJar, "--manifest", $manualManifest, "--min-sdk-version", $MinSdk, "--target-sdk-version", $TargetSdk, "--version-code", "1", "--version-name", "1.0") `
    "aapt2 link failed for OpenAI Realtime APK."

Copy-Item -LiteralPath (Join-Path $dexDir "classes.dex") -Destination (Join-Path $packagingDir "classes.dex")

Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$archive = [System.IO.Compression.ZipFile]::Open($unsignedApk, [System.IO.Compression.ZipArchiveMode]::Update)
try {
    foreach ($file in Get-ChildItem -LiteralPath $packagingDir -File -Recurse) {
        $relative = $file.FullName.Substring($packagingDir.Length + 1).Replace('\', '/')
        [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive, $file.FullName, $relative, [System.IO.Compression.CompressionLevel]::Optimal) | Out-Null
    }
} finally {
    $archive.Dispose()
}

Invoke-ExternalCommand `
    $zipalign `
    @("-f", "4", $unsignedApk, $alignedApk) `
    "zipalign failed for OpenAI Realtime APK."

if (-not (Test-Path -LiteralPath $keystore)) {
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $keystore) | Out-Null
    Invoke-ExternalCommand `
        $keytool `
        @("-genkeypair", "-keystore", $keystore, "-storepass", "android", "-keypass", "android", "-alias", "androiddebugkey", "-keyalg", "RSA", "-keysize", "2048", "-validity", "10000", "-dname", "CN=Android Debug,O=Android,C=US", "-noprompt") `
        "debug keystore generation failed."
}

Invoke-ExternalCommand `
    $apksigner `
    @("sign", "--ks", $keystore, "--ks-pass", "pass:android", "--key-pass", "pass:android", "--out", $OutputApk, $alignedApk) `
    "apksigner failed for OpenAI Realtime APK."

Invoke-ExternalCommand `
    $apksigner `
    @("verify", "--verbose", $OutputApk) `
    "apksigner verify failed for OpenAI Realtime APK."

Write-Output "OpenAI Realtime APK built: $OutputApk"

if ($Install) {
    Invoke-ExternalCommand `
        $adb `
        @("install", "-r", $OutputApk) `
        "adb install failed."
}
