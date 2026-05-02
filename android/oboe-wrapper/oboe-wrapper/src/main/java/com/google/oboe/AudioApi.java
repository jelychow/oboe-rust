package com.google.oboe;

public enum AudioApi {
    UNSPECIFIED(0),
    AAUDIO(1),
    OPENSL_ES(2);

    final int nativeValue;

    AudioApi(int nativeValue) {
        this.nativeValue = nativeValue;
    }
}
