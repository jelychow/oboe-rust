package com.example.minimaloboe;

import android.app.Application;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

public final class AudioPlayer implements Application.ActivityLifecycleCallbacks {
    public interface Listener {
        void onPlayerStateChanged(PlayerState state);
    }

    public enum PlayerState {
        NO_RESULT_YET,
        STARTED,
        STOPPED,
        UNKNOWN
    }

    private static final AudioPlayer INSTANCE = new AudioPlayer();

    static {
        System.loadLibrary("minimaloboe_rust");
    }

    private final ExecutorService executor = Executors.newSingleThreadExecutor();
    private Listener listener;
    private PlayerState state = PlayerState.NO_RESULT_YET;
    private int lastResultCode;

    private AudioPlayer() {}

    public static AudioPlayer getInstance() {
        return INSTANCE;
    }

    public synchronized PlayerState getState() {
        return state;
    }

    public synchronized int getLastResultCode() {
        return lastResultCode;
    }

    public synchronized void setListener(Listener listener) {
        this.listener = listener;
        if (listener != null) {
            listener.onPlayerStateChanged(state);
        }
    }

    public void setPlaybackEnabled(final boolean enabled) {
        executor.execute(
                new Runnable() {
                    @Override
                    public void run() {
                        int result =
                                enabled ? startAudioStreamNative() : stopAudioStreamNative();
                        PlayerState nextState;
                        if (result == 0) {
                            nextState = enabled ? PlayerState.STARTED : PlayerState.STOPPED;
                        } else {
                            nextState = PlayerState.UNKNOWN;
                        }
                        updateState(nextState, result);
                    }
                });
    }

    public void release() {
        setPlaybackEnabled(false);
        executor.shutdown();
    }

    private synchronized void updateState(PlayerState nextState, int resultCode) {
        state = nextState;
        lastResultCode = resultCode;
        if (listener != null) {
            listener.onPlayerStateChanged(nextState);
        }
    }

    @Override
    public void onActivityStopped(android.app.Activity activity) {
        setPlaybackEnabled(false);
    }

    @Override
    public void onActivityCreated(android.app.Activity activity, android.os.Bundle savedInstanceState) {}

    @Override
    public void onActivityStarted(android.app.Activity activity) {}

    @Override
    public void onActivityResumed(android.app.Activity activity) {}

    @Override
    public void onActivityPaused(android.app.Activity activity) {}

    @Override
    public void onActivitySaveInstanceState(android.app.Activity activity, android.os.Bundle outState) {}

    @Override
    public void onActivityDestroyed(android.app.Activity activity) {}

    private static native int startAudioStreamNative();

    private static native int stopAudioStreamNative();
}
