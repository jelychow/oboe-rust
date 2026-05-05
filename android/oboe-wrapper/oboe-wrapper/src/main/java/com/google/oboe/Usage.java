package com.google.oboe;

public enum Usage {
    UNSPECIFIED(0),
    MEDIA(1),
    VOICE_COMMUNICATION(2),
    VOICE_COMMUNICATION_SIGNALLING(3),
    ALARM(4),
    NOTIFICATION(5),
    NOTIFICATION_RINGTONE(6),
    NOTIFICATION_EVENT(10),
    ASSISTANCE_ACCESSIBILITY(11),
    ASSISTANCE_NAVIGATION_GUIDANCE(12),
    ASSISTANCE_SONIFICATION(13),
    GAME(14),
    ASSISTANT(16);

    final int nativeValue;

    Usage(int nativeValue) {
        this.nativeValue = nativeValue;
    }
}
