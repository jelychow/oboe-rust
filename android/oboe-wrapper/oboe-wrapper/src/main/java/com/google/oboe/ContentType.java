package com.google.oboe;

public enum ContentType {
    UNSPECIFIED(0),
    SPEECH(1),
    MUSIC(2),
    MOVIE(3),
    SONIFICATION(4);

    final int nativeValue;

    ContentType(int nativeValue) {
        this.nativeValue = nativeValue;
    }
}
