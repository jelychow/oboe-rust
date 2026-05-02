package com.google.oboe;

import android.test.InstrumentationTestCase;

public final class AudioStreamSmokeTest extends InstrumentationTestCase {
    public void testNativeLibraryLoads() {
        assertEquals(1, AudioStream.nativeVersionCode());
    }

    public void testAAudioOutputLifecycle() {
        AudioStream stream = new AudioStreamBuilder()
                .setAudioApi(AudioApi.AAUDIO)
                .setDirection(AudioDirection.OUTPUT)
                .setSampleRate(48000)
                .setChannelCount(2)
                .setFormat(AudioFormat.FLOAT)
                .setPerformanceMode(PerformanceMode.LOW_LATENCY)
                .openStream();
        try {
            assertEquals(0, stream.requestStart());
            assertEquals(0, stream.requestStop());
        } finally {
            stream.close();
        }
    }

    public void testOpenSlOutputLifecycle() {
        AudioStream stream = new AudioStreamBuilder()
                .setAudioApi(AudioApi.OPENSL_ES)
                .openStream();
        try {
            assertEquals(0, stream.requestStart());
            assertEquals(0, stream.requestStop());
        } finally {
            stream.close();
        }
    }
}
