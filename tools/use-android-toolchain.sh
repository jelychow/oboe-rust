#!/usr/bin/env bash

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

export JAVA_HOME="$REPO_ROOT/.local/jdk-17"
export ANDROID_SDK_ROOT="$REPO_ROOT/.local/android-sdk"
export ANDROID_HOME="$ANDROID_SDK_ROOT"
export ANDROID_USER_HOME="$REPO_ROOT/.local/android-user-home"
export GRADLE_USER_HOME="$REPO_ROOT/.local/gradle-home"
export HOME="$REPO_ROOT/.local/home"
export PATH="$JAVA_HOME/bin:$ANDROID_SDK_ROOT/platform-tools:$ANDROID_SDK_ROOT/cmdline-tools/latest/bin:$REPO_ROOT/.local/gradle-8.7/bin:$PATH"
