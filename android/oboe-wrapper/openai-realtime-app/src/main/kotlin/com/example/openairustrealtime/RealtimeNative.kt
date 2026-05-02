package com.example.openairustrealtime

object RealtimeNative {
    init {
        System.loadLibrary("openai_realtime_jni")
    }

    @JvmStatic external fun startNative(apiKey: String, model: String, instructions: String): Int

    @JvmStatic external fun stopNative(): Int

    @JvmStatic external fun playPcmNative(pcm: ByteArray): Int

    @JvmStatic external fun stopNativeAudio(): Int

    @JvmStatic external fun startWavRecordingNative(path: String): Int

    @JvmStatic external fun stopWavRecordingNative(): Int

    @JvmStatic external fun nativeAudioErrorNative(): String?

    @JvmStatic external fun statusNative(): String?

    @JvmStatic external fun transcriptNative(): String?

    @JvmStatic external fun lastErrorNative(): String?

    @JvmStatic external fun statsNative(): String?
}
