package com.example.openairustrealtime.core.util

object ReleaseLogPolicy {
    fun shouldLogDebug(isDebugBuild: Boolean): Boolean = isDebugBuild

    fun shouldLogInfo(isDebugBuild: Boolean): Boolean = isDebugBuild

    fun shouldLogWarn(isDebugBuild: Boolean): Boolean = true

    fun shouldLogError(isDebugBuild: Boolean): Boolean = true
}
