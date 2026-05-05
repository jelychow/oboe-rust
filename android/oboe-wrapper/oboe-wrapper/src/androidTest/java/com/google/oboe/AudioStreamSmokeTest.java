package com.google.oboe;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;
import static org.junit.Assert.fail;

import android.Manifest;
import androidx.test.ext.junit.runners.AndroidJUnit4;
import androidx.test.rule.GrantPermissionRule;
import org.junit.Rule;
import org.junit.Test;
import org.junit.runner.RunWith;

@RunWith(AndroidJUnit4.class)
public final class AudioStreamSmokeTest {
    @Rule
    public final GrantPermissionRule recordAudioPermission =
            GrantPermissionRule.grant(Manifest.permission.RECORD_AUDIO);

    @Test
    public void nativeLibraryLoads() {
        assertEquals(AudioStream.EXPECTED_NATIVE_VERSION_CODE, AudioStream.nativeVersionCode());
    }

    @Test
    public void aaudioOutputLifecycle() {
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

    @Test
    public void aaudioOutputWritesFloatPcm() {
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

    @Test
    public void aaudioInputReadsFloatPcm() {
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

    @Test
    public void aaudioInputAllocatesSessionForCaptureEffects() {
        AudioStream stream = new AudioStreamBuilder()
                .setAudioApi(AudioApi.AAUDIO)
                .setDirection(AudioDirection.INPUT)
                .setInputPreset(InputPreset.VOICE_COMMUNICATION)
                .setSessionId(AudioStreamBuilder.SESSION_ID_ALLOCATE)
                .setSampleRate(24000)
                .setChannelCount(1)
                .setFormat(AudioFormat.FLOAT)
                .setPerformanceMode(PerformanceMode.LOW_LATENCY)
                .openStream();
        try {
            assertTrue("session id should be allocated for Java audio effects", stream.getSessionId() > 0);
        } finally {
            stream.close();
        }
    }

    @Test
    public void openSlOutputLifecycle() {
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

    @Test
    public void aaudioLowLatencyDiagnosticsApis() {
        AudioStream stream = new AudioStreamBuilder()
                .setAudioApi(AudioApi.AAUDIO)
                .setDirection(AudioDirection.OUTPUT)
                .setSampleRate(48000)
                .setChannelCount(1)
                .setFormat(AudioFormat.FLOAT)
                .setSharingMode(SharingMode.EXCLUSIVE)
                .setPerformanceMode(PerformanceMode.LOW_LATENCY)
                .setFramesPerCallback(96)
                .setBufferCapacityInFrames(384)
                .openStream();
        try {
            assertTrue("burst size should be queryable", stream.getFramesPerBurst() >= 0);
            assertTrue(
                    "buffer tuning should return the actual buffer size",
                    stream.setBufferSizeInFrames(192) >= 0);
            assertTrue("buffer size should be queryable", stream.getBufferSizeInFrames() >= 0);
            assertTrue(
                    "buffer capacity should be queryable",
                    stream.getBufferCapacityInFrames() >= 0);
            assertTrue("xrun count should be queryable", stream.getXRunCount() >= 0);
            assertEquals(0, stream.getAndClearLastError());
            try {
                AudioTimestamp timestamp = stream.getTimestamp();
                assertNotNull(timestamp);
            } catch (IllegalStateException expected) {
                assertEquals("native stream timestamp query failed", expected.getMessage());
            }
        } finally {
            stream.close();
        }
    }

    @Test
    public void closeReportsNativeCloseFailureForUnknownHandle() {
        AudioStream stream = new AudioStream(Long.MAX_VALUE);
        try {
            stream.close();
            fail("close should report native close failures");
        } catch (IllegalStateException expected) {
            assertEquals("native stream close failed", expected.getMessage());
        }
    }
}
