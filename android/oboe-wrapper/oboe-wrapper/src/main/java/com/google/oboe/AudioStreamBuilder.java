package com.google.oboe;

public final class AudioStreamBuilder {
    public static final int SESSION_ID_NONE = -1;
    public static final int SESSION_ID_ALLOCATE = 0;

    private AudioApi audioApi = AudioApi.UNSPECIFIED;
    private AudioDirection direction = AudioDirection.OUTPUT;
    private SharingMode sharingMode = SharingMode.SHARED;
    private PerformanceMode performanceMode = PerformanceMode.NONE;
    private Usage usage = Usage.MEDIA;
    private ContentType contentType = ContentType.MUSIC;
    private InputPreset inputPreset = InputPreset.UNSPECIFIED;
    private int sessionId = SESSION_ID_NONE;
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

    public AudioStreamBuilder setInputPreset(InputPreset inputPreset) {
        if (inputPreset == null) {
            throw new IllegalArgumentException("inputPreset must not be null");
        }
        this.inputPreset = inputPreset;
        return this;
    }

    public AudioStreamBuilder setUsage(Usage usage) {
        if (usage == null) {
            throw new IllegalArgumentException("usage must not be null");
        }
        this.usage = usage;
        return this;
    }

    public AudioStreamBuilder setContentType(ContentType contentType) {
        if (contentType == null) {
            throw new IllegalArgumentException("contentType must not be null");
        }
        this.contentType = contentType;
        return this;
    }

    public AudioStreamBuilder setSessionId(int sessionId) {
        if (sessionId < SESSION_ID_NONE) {
            throw new IllegalArgumentException("sessionId must be -1, 0, or positive");
        }
        this.sessionId = sessionId;
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
        AudioStream.ensureNativeVersion();
        long handle = AudioStream.nativeOpen(
                audioApi.nativeValue,
                direction.nativeValue,
                sharingMode.nativeValue,
                performanceMode.nativeValue,
                usage.nativeValue,
                contentType.nativeValue,
                inputPreset.nativeValue,
                sessionId,
                sampleRate,
                channelCount,
                format.nativeValue,
                framesPerCallback,
                bufferCapacityInFrames);
        if (handle == 0) {
            int openError = AudioStream.nativeGetLastOpenError();
            String suffix = openError == 0 ? "" : " code=" + openError;
            throw new IllegalStateException("native stream open failed" + suffix);
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
