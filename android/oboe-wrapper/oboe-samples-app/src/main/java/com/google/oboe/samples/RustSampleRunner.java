package com.google.oboe.samples;

public final class RustSampleRunner {
    public static final int API_AAUDIO = 1;
    public static final int API_OPENSL_ES = 2;

    static {
        System.loadLibrary("oboe_samples_jni");
    }

    private RustSampleRunner() {}

    public static native int nativeSampleCount();

    public static native int nativeRunSample(int sampleId, int audioApi, int durationMillis);
}
