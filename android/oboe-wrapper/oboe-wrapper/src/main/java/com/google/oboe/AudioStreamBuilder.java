package com.google.oboe;

public final class AudioStreamBuilder {
    private AudioApi audioApi = AudioApi.UNSPECIFIED;
    private AudioDirection direction = AudioDirection.OUTPUT;
    private SharingMode sharingMode = SharingMode.SHARED;
    private PerformanceMode performanceMode = PerformanceMode.NONE;
    private int sampleRate;
    private int channelCount = 2;
    private AudioFormat format = AudioFormat.FLOAT;
    private int framesPerCallback;
    private int bufferCapacityInFrames;

    public AudioStreamBuilder setAudioApi(AudioApi audioApi) {
        if (audioApi == null) {
            throw new IllegalArgumentException("audioApi must not be null");
        }
        this.audioApi = audioApi;
        return this;
    }

    public AudioStreamBuilder setDirection(AudioDirection direction) {
        if (direction == null) {
            throw new IllegalArgumentException("direction must not be null");
        }
        this.direction = direction;
        return this;
    }

    public AudioStreamBuilder setSharingMode(SharingMode sharingMode) {
        if (sharingMode == null) {
            throw new IllegalArgumentException("sharingMode must not be null");
        }
        this.sharingMode = sharingMode;
        return this;
    }

    public AudioStreamBuilder setPerformanceMode(PerformanceMode performanceMode) {
        if (performanceMode == null) {
            throw new IllegalArgumentException("performanceMode must not be null");
        }
        this.performanceMode = performanceMode;
        return this;
    }

    public AudioStreamBuilder setSampleRate(int sampleRate) {
        if (sampleRate < 0) {
            throw new IllegalArgumentException("sampleRate must be non-negative");
        }
        this.sampleRate = sampleRate;
        return this;
    }

    public AudioStreamBuilder setChannelCount(int channelCount) {
        if (channelCount <= 0) {
            throw new IllegalArgumentException("channelCount must be positive");
        }
        this.channelCount = channelCount;
        return this;
    }

    public AudioStreamBuilder setFormat(AudioFormat format) {
        if (format == null) {
            throw new IllegalArgumentException("format must not be null");
        }
        this.format = format;
        return this;
    }

    public AudioStreamBuilder setFramesPerCallback(int framesPerCallback) {
        if (framesPerCallback < 0) {
            throw new IllegalArgumentException("framesPerCallback must be non-negative");
        }
        this.framesPerCallback = framesPerCallback;
        return this;
    }

    public AudioStreamBuilder setBufferCapacityInFrames(int bufferCapacityInFrames) {
        if (bufferCapacityInFrames < 0) {
            throw new IllegalArgumentException("bufferCapacityInFrames must be non-negative");
        }
        this.bufferCapacityInFrames = bufferCapacityInFrames;
        return this;
    }

    public AudioStreamBuilder setDataCallback(
            AudioCallback callback, int framesPerDataCallback) {
        if (callback == null) {
            throw new IllegalArgumentException("callback must not be null");
        }
        validateFramesPerDataCallback(framesPerDataCallback);
        throw unsupportedCallbackDispatch();
    }

    public AudioStreamBuilder setPartialDataCallback(
            AudioPartialDataCallback callback, int framesPerDataCallback) {
        if (callback == null) {
            throw new IllegalArgumentException("callback must not be null");
        }
        validateFramesPerDataCallback(framesPerDataCallback);
        throw unsupportedCallbackDispatch();
    }

    public AudioStreamBuilder setPresentationCallback(AudioPresentationCallback callback) {
        if (callback == null) {
            throw new IllegalArgumentException("callback must not be null");
        }
        throw unsupportedCallbackDispatch();
    }

    public AudioStreamBuilder setRoutingCallback(AudioRoutingCallback callback) {
        if (callback == null) {
            throw new IllegalArgumentException("callback must not be null");
        }
        throw unsupportedCallbackDispatch();
    }

    public AudioStream openStream() {
        long handle = AudioStream.nativeOpen(
                audioApi.nativeValue,
                direction.nativeValue,
                sharingMode.nativeValue,
                performanceMode.nativeValue,
                sampleRate,
                channelCount,
                format.nativeValue,
                framesPerCallback,
                bufferCapacityInFrames);
        if (handle == 0) {
            throw new IllegalStateException("native stream open failed");
        }
        return new AudioStream(handle);
    }

    private static void validateFramesPerDataCallback(int framesPerDataCallback) {
        if (framesPerDataCallback < 0) {
            throw new IllegalArgumentException("framesPerDataCallback must be non-negative");
        }
    }

    private static UnsupportedOperationException unsupportedCallbackDispatch() {
        return new UnsupportedOperationException(
                "Java audio callbacks are not wired to the Rust audio callback thread yet");
    }
}
