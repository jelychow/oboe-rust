package com.google.oboe;

public interface AudioCallback {
    int onAudioReady(AudioStream stream, float[] audioData, int numFrames);

    void onError(AudioStream stream, int error);
}
