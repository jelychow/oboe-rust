package com.google.oboe;

public final class AudioStream implements AutoCloseable {
    static {
        System.loadLibrary("oboe_jni");
    }

    private long nativeHandle;

    AudioStream(long nativeHandle) {
        this.nativeHandle = nativeHandle;
    }

    public static native int nativeVersionCode();

    static native long nativeOpen(int audioApi);

    private static native int nativeRequestStart(long handle);

    private static native int nativeRequestStop(long handle);

    private static native int nativeGetState(long handle);

    private static native int nativeClose(long handle);

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
}
