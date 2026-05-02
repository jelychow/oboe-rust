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

    public void testAAudioOutputWritesFloatPcm() {
        AudioStream stream = new AudioStreamBuilder()
                .setAudioApi(AudioApi.AAUDIO)
                .setDirection(AudioDirection.OUTPUT)
                .setSampleRate(24000)
                .setChannelCount(1)
                .setFormat(AudioFormat.FLOAT)
                .setPerformanceMode(PerformanceMode.LOW_LATENCY)
                .openStream();
        try {
            assertEquals(0, stream.requestStart());
            int written = stream.writeFloat(new float[96], 0, 96, 100_000_000L);
            assertTrue("writeFloat should not return an error", written >= 0);
            assertEquals(0, stream.requestStop());
        } finally {
            stream.close();
        }
    }

    public void testAAudioInputReadsFloatPcm() {
        AudioStream stream = new AudioStreamBuilder()
                .setAudioApi(AudioApi.AAUDIO)
                .setDirection(AudioDirection.INPUT)
                .setSampleRate(24000)
                .setChannelCount(1)
                .setFormat(AudioFormat.FLOAT)
                .setPerformanceMode(PerformanceMode.LOW_LATENCY)
                .openStream();
        try {
            assertEquals(0, stream.requestStart());
            int read = stream.readFloat(new float[96], 0, 96, 100_000_000L);
            assertTrue("readFloat should not return an error", read >= 0);
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
