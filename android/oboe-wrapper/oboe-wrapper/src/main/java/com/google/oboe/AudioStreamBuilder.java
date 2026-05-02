package com.google.oboe;

public final class AudioStreamBuilder {
    private AudioApi audioApi = AudioApi.UNSPECIFIED;

    public AudioStreamBuilder setAudioApi(AudioApi audioApi) {
        if (audioApi == null) {
            throw new IllegalArgumentException("audioApi must not be null");
        }
        this.audioApi = audioApi;
        return this;
    }

    public AudioStream openStream() {
        long handle = AudioStream.nativeOpen(audioApi.nativeValue);
        if (handle == 0) {
            throw new IllegalStateException("native stream open failed");
        }
        return new AudioStream(handle);
    }
}
