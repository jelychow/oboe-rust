package com.google.oboe;

public final class PlaybackParameters {
    public static final int FALLBACK_DEFAULT = 0;
    public static final int FALLBACK_MUTE = 1;
    public static final int FALLBACK_FAIL = 2;

    public static final int STRETCH_DEFAULT = 0;
    public static final int STRETCH_VOICE = 1;

    public final int fallbackMode;
    public final int stretchMode;
    public final float pitch;
    public final float speed;

    public PlaybackParameters(int fallbackMode, int stretchMode, float pitch, float speed) {
        validateMode(fallbackMode, FALLBACK_DEFAULT, FALLBACK_FAIL, "fallbackMode");
        validateMode(stretchMode, STRETCH_DEFAULT, STRETCH_VOICE, "stretchMode");
        if (!Float.isFinite(pitch) || pitch < 0.25f || pitch > 4.0f) {
            throw new IllegalArgumentException("pitch must be finite and in [0.25, 4.0]");
        }
        if (!Float.isFinite(speed) || speed < 0.01f || speed > 20.0f) {
            throw new IllegalArgumentException("speed must be finite and in [0.01, 20.0]");
        }
        this.fallbackMode = fallbackMode;
        this.stretchMode = stretchMode;
        this.pitch = pitch;
        this.speed = speed;
    }

    public static PlaybackParameters defaults() {
        return new PlaybackParameters(FALLBACK_DEFAULT, STRETCH_DEFAULT, 1.0f, 1.0f);
    }

    private static void validateMode(int value, int min, int max, String name) {
        if (value < min || value > max) {
            throw new IllegalArgumentException(name + " is out of range");
        }
    }
}
