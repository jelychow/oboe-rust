package com.google.oboe;

public enum AudioDirection {
    INPUT(0),
    OUTPUT(1);

    final int nativeValue;

    AudioDirection(int nativeValue) {
        this.nativeValue = nativeValue;
    }
}
