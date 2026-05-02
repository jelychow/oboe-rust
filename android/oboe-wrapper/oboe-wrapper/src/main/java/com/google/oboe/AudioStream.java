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

    static native long nativeOpen(int audioApi);

    private static native int nativeRequestStart(long handle);

    private static native int nativeRequestStop(long handle);

    private static native int nativeGetState(long handle);

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

    public int setDataCallback(AudioCallback callback, int framesPerDataCallback) {
        ensureOpen();
        if (callback == null) {
            throw new IllegalArgumentException("callback must not be null");
        }
        validateFramesPerDataCallback(framesPerDataCallback);
        int result = nativeSetCallbackConfig(
                nativeHandle,
                true,
                false,
                presentationCallback != null,
                routingCallback != null,
                framesPerDataCallback);
        if (result == 0) {
            dataCallback = callback;
            partialDataCallback = null;
            this.framesPerDataCallback = framesPerDataCallback;
        }
        return result;
    }

    public int setPartialDataCallback(
            AudioPartialDataCallback callback, int framesPerDataCallback) {
        ensureOpen();
        if (callback == null) {
            throw new IllegalArgumentException("callback must not be null");
        }
        validateFramesPerDataCallback(framesPerDataCallback);
        int result = nativeSetCallbackConfig(
                nativeHandle,
                false,
                true,
                presentationCallback != null,
                routingCallback != null,
                framesPerDataCallback);
        if (result == 0) {
            dataCallback = null;
            partialDataCallback = callback;
            this.framesPerDataCallback = framesPerDataCallback;
        }
        return result;
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
        int result = nativeSetCallbackConfig(
                nativeHandle,
                dataCallback != null,
                partialDataCallback != null,
                true,
                routingCallback != null,
                framesPerDataCallback);
        if (result == 0) {
            presentationCallback = callback;
        }
        return result;
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
        int result = nativeSetCallbackConfig(
                nativeHandle,
                dataCallback != null,
                partialDataCallback != null,
                presentationCallback != null,
                true,
                framesPerDataCallback);
        if (result == 0) {
            routingCallback = callback;
        }
        return result;
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
}
