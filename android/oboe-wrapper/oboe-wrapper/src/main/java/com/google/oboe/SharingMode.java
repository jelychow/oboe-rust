package com.google.oboe;

public enum SharingMode {
    SHARED(0),
    EXCLUSIVE(1);

    final int nativeValue;

    SharingMode(int nativeValue) {
        this.nativeValue = nativeValue;
    }
}
