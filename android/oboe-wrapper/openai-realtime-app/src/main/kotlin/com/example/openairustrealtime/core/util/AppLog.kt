package com.example.openairustrealtime.core.util

import android.util.Log
import com.example.openairustrealtime.BuildConfig

object AppLog {
    fun d(tag: String, message: String) {
        if (ReleaseLogPolicy.shouldLogDebug(BuildConfig.DEBUG)) {
            Log.d(tag, message)
        }
    }

    fun i(tag: String, message: String) {
        if (ReleaseLogPolicy.shouldLogInfo(BuildConfig.DEBUG)) {
            Log.i(tag, message)
        }
    }

    fun w(tag: String, message: String, throwable: Throwable? = null) {
        if (throwable == null) {
            Log.w(tag, message)
        } else {
            Log.w(tag, message, throwable)
        }
    }
}
