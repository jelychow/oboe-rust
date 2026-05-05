package com.google.oboe;

public enum AudioFormat {
    UNSPECIFIED(0),
    I16(1),
    I24(2),
    I32(3),
    FLOAT(4);

    final int nativeValue;

    AudioFormat(int nativeValue) {
        this.nativeValue = nativeValue;
    }
}
