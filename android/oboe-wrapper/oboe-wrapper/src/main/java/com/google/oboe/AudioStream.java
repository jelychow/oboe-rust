package com.google.oboe;

public final class AudioStream implements AutoCloseable {
    static {
        System.loadLibrary("oboe_jni");
    }

    private long nativeHandle;
    private AudioCallback dataCallback;
    private AudioPartialDataCallback partialDataCallback;
    private AudioPresentationCallback presentationCallback;
    private AudioRoutingCallback routingCallback;
    private int framesPerDataCallback;

    AudioStream(long nativeHandle) {
        this.nativeHandle = nativeHandle;
    }

    public static native int nativeVersionCode();

    static native long nativeOpen(
            int audioApi,
            int direction,
            int sharingMode,
            int performanceMode,
            int sampleRate,
            int channelCount,
            int format,
            int framesPerCallback,
            int bufferCapacityInFrames);

    private static native int nativeRequestStart(long handle);

    private static native int nativeRequestStop(long handle);

    private static native int nativeGetState(long handle);

    private static native int nativeGetTimestamp(long handle, long[] out);

    private static native long nativeGetFramesRead(long handle);

    private static native long nativeGetFramesWritten(long handle);

    private static native int nativeGetXRunCount(long handle);

    private static native int nativeGetFramesPerBurst(long handle);

    private static native int nativeGetBufferSizeInFrames(long handle);

    private static native int nativeSetBufferSizeInFrames(long handle, int frames);

    private static native int nativeGetBufferCapacityInFrames(long handle);

    private static native int nativeGetAndClearLastError(long handle);

    private static native int nativeClose(long handle);

    private static native int nativeSetCallbackConfig(
            long handle,
            boolean dataCallback,
            boolean partialDataCallback,
            boolean presentationCallback,
            boolean routingCallback,
            int framesPerDataCallback);

    private static native int nativeSetOffloadDelayPadding(
            long handle, int delayInFrames, int paddingInFrames);

    private static native int nativeSetOffloadEndOfStream(long handle);

    private static native int nativeSetPlaybackParameters(
            long handle, int fallbackMode, int stretchMode, float pitch, float speed);

    private static native int nativeSetPresentationTimestamp(
            long handle, long framePosition, long timestampNanos);

    private static native int nativeSetRouteDeviceId(long handle, int deviceId);

    private static native int nativeWriteFloat(
            long handle, float[] audioData, int offset, int sampleCount, long timeoutNanos);

    private static native int nativeReadFloat(
            long handle, float[] audioData, int offset, int sampleCount, long timeoutNanos);

    public int requestStart() {
        ensureOpen();
        return nativeRequestStart(nativeHandle);
    }

    public int requestStop() {
        ensureOpen();
        return nativeRequestStop(nativeHandle);
    }

    public int getState() {
        ensureOpen();
        return nativeGetState(nativeHandle);
    }

    public AudioTimestamp getTimestamp() {
        ensureOpen();
        long[] timestamp = new long[2];
        int result = nativeGetTimestamp(nativeHandle, timestamp);
        if (result != 0) {
            throw new IllegalStateException("native stream timestamp query failed");
        }
        return new AudioTimestamp(timestamp[0], timestamp[1]);
    }

    public long getFramesRead() {
        ensureOpen();
        return nativeGetFramesRead(nativeHandle);
    }

    public long getFramesWritten() {
        ensureOpen();
        return nativeGetFramesWritten(nativeHandle);
    }

    public int getXRunCount() {
        ensureOpen();
        return nativeGetXRunCount(nativeHandle);
    }

    public int getFramesPerBurst() {
        ensureOpen();
        return nativeGetFramesPerBurst(nativeHandle);
    }

    public int getBufferSizeInFrames() {
        ensureOpen();
        return nativeGetBufferSizeInFrames(nativeHandle);
    }

    public int setBufferSizeInFrames(int frames) {
        ensureOpen();
        if (frames <= 0) {
            throw new IllegalArgumentException("frames must be positive");
        }
        return nativeSetBufferSizeInFrames(nativeHandle, frames);
    }

    public int getBufferCapacityInFrames() {
        ensureOpen();
        return nativeGetBufferCapacityInFrames(nativeHandle);
    }

    public int getAndClearLastError() {
        ensureOpen();
        return nativeGetAndClearLastError(nativeHandle);
    }

    public int setDataCallback(AudioCallback callback, int framesPerDataCallback) {
        ensureOpen();
        if (callback == null) {
            throw new IllegalArgumentException("callback must not be null");
        }
        validateFramesPerDataCallback(framesPerDataCallback);
        throw unsupportedCallbackDispatch();
    }

    public int setPartialDataCallback(
            AudioPartialDataCallback callback, int framesPerDataCallback) {
        ensureOpen();
        if (callback == null) {
            throw new IllegalArgumentException("callback must not be null");
        }
        validateFramesPerDataCallback(framesPerDataCallback);
        throw unsupportedCallbackDispatch();
    }

    public int clearDataCallbacks() {
        ensureOpen();
        int result = nativeSetCallbackConfig(
                nativeHandle,
                false,
                false,
                presentationCallback != null,
                routingCallback != null,
                framesPerDataCallback);
        if (result == 0) {
            dataCallback = null;
            partialDataCallback = null;
        }
        return result;
    }

    public int setPresentationCallback(AudioPresentationCallback callback) {
        ensureOpen();
        if (callback == null) {
            throw new IllegalArgumentException("callback must not be null");
        }
        throw unsupportedCallbackDispatch();
    }

    public int clearPresentationCallback() {
        ensureOpen();
        int result = nativeSetCallbackConfig(
                nativeHandle,
                dataCallback != null,
                partialDataCallback != null,
                false,
                routingCallback != null,
                framesPerDataCallback);
        if (result == 0) {
            presentationCallback = null;
        }
        return result;
    }

    public int setRoutingCallback(AudioRoutingCallback callback) {
        ensureOpen();
        if (callback == null) {
            throw new IllegalArgumentException("callback must not be null");
        }
        throw unsupportedCallbackDispatch();
    }

    public int clearRoutingCallback() {
        ensureOpen();
        int result = nativeSetCallbackConfig(
                nativeHandle,
                dataCallback != null,
                partialDataCallback != null,
                presentationCallback != null,
                false,
                framesPerDataCallback);
        if (result == 0) {
            routingCallback = null;
        }
        return result;
    }

    public int setOffloadDelayPadding(int delayInFrames, int paddingInFrames) {
        ensureOpen();
        if (delayInFrames < 0 || paddingInFrames < 0) {
            throw new IllegalArgumentException("offload delay and padding must be non-negative");
        }
        return nativeSetOffloadDelayPadding(nativeHandle, delayInFrames, paddingInFrames);
    }

    public int setOffloadEndOfStream() {
        ensureOpen();
        return nativeSetOffloadEndOfStream(nativeHandle);
    }

    public int setPlaybackParameters(PlaybackParameters parameters) {
        ensureOpen();
        if (parameters == null) {
            throw new IllegalArgumentException("parameters must not be null");
        }
        return nativeSetPlaybackParameters(
                nativeHandle,
                parameters.fallbackMode,
                parameters.stretchMode,
                parameters.pitch,
                parameters.speed);
    }

    public int setPresentationTimestamp(long framePosition, long timestampNanos) {
        ensureOpen();
        if (framePosition < 0 || timestampNanos < 0) {
            throw new IllegalArgumentException(
                    "presentation frame position and timestamp must be non-negative");
        }
        return nativeSetPresentationTimestamp(nativeHandle, framePosition, timestampNanos);
    }

    public int setRouteDeviceId(int deviceId) {
        ensureOpen();
        if (deviceId < 0) {
            throw new IllegalArgumentException("deviceId must be non-negative");
        }
        return nativeSetRouteDeviceId(nativeHandle, deviceId);
    }

    public int writeFloat(
            float[] audioData, int offset, int sampleCount, long timeoutNanos) {
        ensureOpen();
        validateFloatIo(audioData, offset, sampleCount, timeoutNanos);
        return nativeWriteFloat(nativeHandle, audioData, offset, sampleCount, timeoutNanos);
    }

    public int readFloat(
            float[] audioData, int offset, int sampleCount, long timeoutNanos) {
        ensureOpen();
        validateFloatIo(audioData, offset, sampleCount, timeoutNanos);
        return nativeReadFloat(nativeHandle, audioData, offset, sampleCount, timeoutNanos);
    }

    @Override
    public void close() {
        if (nativeHandle != 0) {
            if (nativeClose(nativeHandle) != 0) {
                throw new IllegalStateException("native stream close failed");
            }
            nativeHandle = 0;
        }
    }

    private void ensureOpen() {
        if (nativeHandle == 0) {
            throw new IllegalStateException("stream is closed");
        }
    }

    private static void validateFramesPerDataCallback(int framesPerDataCallback) {
        if (framesPerDataCallback < 0) {
            throw new IllegalArgumentException("framesPerDataCallback must be non-negative");
        }
    }

    private static void validateFloatIo(
            float[] audioData, int offset, int sampleCount, long timeoutNanos) {
        if (audioData == null) {
            throw new IllegalArgumentException("audioData must not be null");
        }
        if (offset < 0) {
            throw new IllegalArgumentException("offset must be non-negative");
        }
        if (sampleCount < 0) {
            throw new IllegalArgumentException("sampleCount must be non-negative");
        }
        if (timeoutNanos < 0) {
            throw new IllegalArgumentException("timeoutNanos must be non-negative");
        }
        if (offset > audioData.length || sampleCount > audioData.length - offset) {
            throw new IllegalArgumentException("audioData range is outside the array bounds");
        }
    }

    private static UnsupportedOperationException unsupportedCallbackDispatch() {
        return new UnsupportedOperationException(
                "Java audio callbacks are not wired to the Rust audio callback thread yet");
    }
}
