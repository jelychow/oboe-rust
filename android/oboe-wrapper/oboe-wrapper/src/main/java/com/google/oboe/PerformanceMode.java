package com.google.oboe;

public enum PerformanceMode {
    NONE(0),
    POWER_SAVING(1),
    LOW_LATENCY(2);

    final int nativeValue;

    PerformanceMode(int nativeValue) {
        this.nativeValue = nativeValue;
    }
}
