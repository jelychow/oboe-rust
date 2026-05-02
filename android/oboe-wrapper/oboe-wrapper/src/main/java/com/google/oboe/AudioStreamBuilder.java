package com.google.oboe;

public final class AudioStreamBuilder {
    private AudioApi audioApi = AudioApi.UNSPECIFIED;
    private AudioCallback dataCallback;
    private AudioPartialDataCallback partialDataCallback;
    private AudioPresentationCallback presentationCallback;
    private AudioRoutingCallback routingCallback;
    private int framesPerDataCallback;

    public AudioStreamBuilder setAudioApi(AudioApi audioApi) {
        if (audioApi == null) {
            throw new IllegalArgumentException("audioApi must not be null");
        }
        this.audioApi = audioApi;
        return this;
    }

    public AudioStreamBuilder setDataCallback(
            AudioCallback callback, int framesPerDataCallback) {
        if (callback == null) {
            throw new IllegalArgumentException("callback must not be null");
        }
        validateFramesPerDataCallback(framesPerDataCallback);
        this.dataCallback = callback;
        this.partialDataCallback = null;
        this.framesPerDataCallback = framesPerDataCallback;
        return this;
    }

    public AudioStreamBuilder setPartialDataCallback(
            AudioPartialDataCallback callback, int framesPerDataCallback) {
        if (callback == null) {
            throw new IllegalArgumentException("callback must not be null");
        }
        validateFramesPerDataCallback(framesPerDataCallback);
        this.dataCallback = null;
        this.partialDataCallback = callback;
        this.framesPerDataCallback = framesPerDataCallback;
        return this;
    }

    public AudioStreamBuilder setPresentationCallback(AudioPresentationCallback callback) {
        if (callback == null) {
            throw new IllegalArgumentException("callback must not be null");
        }
        this.presentationCallback = callback;
        return this;
    }

    public AudioStreamBuilder setRoutingCallback(AudioRoutingCallback callback) {
        if (callback == null) {
            throw new IllegalArgumentException("callback must not be null");
        }
        this.routingCallback = callback;
        return this;
    }

    public AudioStream openStream() {
        long handle = AudioStream.nativeOpen(audioApi.nativeValue);
        if (handle == 0) {
            throw new IllegalStateException("native stream open failed");
        }
        AudioStream stream = new AudioStream(handle);
        try {
            applyCallbacks(stream);
        } catch (RuntimeException error) {
            try {
                stream.close();
            } catch (RuntimeException closeError) {
                error.addSuppressed(closeError);
            }
            throw error;
        }
        return stream;
    }

    private void applyCallbacks(AudioStream stream) {
        if (dataCallback != null) {
            requireNativeOk(
                    stream.setDataCallback(dataCallback, framesPerDataCallback),
                    "setDataCallback");
        }
        if (partialDataCallback != null) {
            requireNativeOk(
                    stream.setPartialDataCallback(partialDataCallback, framesPerDataCallback),
                    "setPartialDataCallback");
        }
        if (presentationCallback != null) {
            requireNativeOk(
                    stream.setPresentationCallback(presentationCallback),
                    "setPresentationCallback");
        }
        if (routingCallback != null) {
            requireNativeOk(stream.setRoutingCallback(routingCallback), "setRoutingCallback");
        }
    }

    private static void requireNativeOk(int result, String operation) {
        if (result != 0) {
            throw new IllegalStateException(operation + " failed");
        }
    }

    private static void validateFramesPerDataCallback(int framesPerDataCallback) {
        if (framesPerDataCallback < 0) {
            throw new IllegalArgumentException("framesPerDataCallback must be non-negative");
        }
    }
}
