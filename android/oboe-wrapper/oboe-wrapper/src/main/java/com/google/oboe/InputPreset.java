package com.google.oboe;

public enum InputPreset {
    UNSPECIFIED(0),
    GENERIC(1),
    CAMCORDER(5),
    VOICE_RECOGNITION(6),
    VOICE_COMMUNICATION(7),
    UNPROCESSED(9),
    VOICE_PERFORMANCE(10);

    final int nativeValue;

    InputPreset(int nativeValue) {
        this.nativeValue = nativeValue;
    }
}
