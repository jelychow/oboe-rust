package com.google.oboe;

public interface AudioPartialDataCallback {
    int onAudioReady(AudioStream stream, float[] audioData, int numFrames);
}
