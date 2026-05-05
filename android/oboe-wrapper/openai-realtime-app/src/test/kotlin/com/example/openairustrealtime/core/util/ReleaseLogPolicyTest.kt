package com.example.openairustrealtime.core.util

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ReleaseLogPolicyTest {
    @Test
    fun suppressesDebugAndInfoLogsInReleaseBuilds() {
        assertFalse(ReleaseLogPolicy.shouldLogDebug(isDebugBuild = false))
        assertFalse(ReleaseLogPolicy.shouldLogInfo(isDebugBuild = false))
    }

    @Test
    fun keepsWarningsAndErrorsInReleaseBuilds() {
        assertTrue(ReleaseLogPolicy.shouldLogWarn(isDebugBuild = false))
        assertTrue(ReleaseLogPolicy.shouldLogError(isDebugBuild = false))
    }

    @Test
    fun allowsVerboseOperationalLogsInDebugBuilds() {
        assertTrue(ReleaseLogPolicy.shouldLogDebug(isDebugBuild = true))
        assertTrue(ReleaseLogPolicy.shouldLogInfo(isDebugBuild = true))
    }
}
