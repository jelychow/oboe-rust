package com.google.oboe;

public interface AudioRoutingCallback {
    void onRoutingChanged(AudioStream stream, int[] deviceIds);
}
