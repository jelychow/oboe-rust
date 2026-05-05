package com.google.oboe;

public final class AudioTimestamp {
    public final long framePosition;
    public final long timeNanoseconds;

    public AudioTimestamp(long framePosition, long timeNanoseconds) {
        this.framePosition = framePosition;
        this.timeNanoseconds = timeNanoseconds;
    }
}
