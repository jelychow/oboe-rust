package com.example.minimaloboe;

import android.app.Activity;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.view.Gravity;
import android.view.View;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.TextView;

public final class MainActivity extends Activity implements AudioPlayer.Listener {
    private final Handler mainHandler = new Handler(Looper.getMainLooper());
    private TextView status;
    private Button startButton;
    private Button stopButton;
    private AudioPlayer audioPlayer;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        audioPlayer = AudioPlayer.getInstance();
        getApplication().registerActivityLifecycleCallbacks(audioPlayer);
        audioPlayer.setListener(this);
        setContentView(createContentView());
        updateUi(audioPlayer.getState());
    }

    @Override
    protected void onDestroy() {
        audioPlayer.setListener(null);
        getApplication().unregisterActivityLifecycleCallbacks(audioPlayer);
        super.onDestroy();
    }

    @Override
    public void onPlayerStateChanged(final AudioPlayer.PlayerState state) {
        mainHandler.post(
                new Runnable() {
                    @Override
                    public void run() {
                        updateUi(state);
                    }
                });
    }

    private LinearLayout createContentView() {
        LinearLayout root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setGravity(Gravity.CENTER);
        root.setPadding(48, 48, 48, 48);

        TextView title = new TextView(this);
        title.setText("Minimal Oboe!");
        title.setTextSize(24.0f);
        root.addView(title);

        LinearLayout controls = new LinearLayout(this);
        controls.setOrientation(LinearLayout.HORIZONTAL);
        controls.setGravity(Gravity.CENTER);
        controls.setPadding(0, 24, 0, 24);

        startButton = new Button(this);
        startButton.setAllCaps(false);
        startButton.setText("Start Audio");
        startButton.setOnClickListener(
                new View.OnClickListener() {
                    @Override
                    public void onClick(View view) {
                        audioPlayer.setPlaybackEnabled(true);
                    }
                });
        controls.addView(startButton);

        stopButton = new Button(this);
        stopButton.setAllCaps(false);
        stopButton.setText("Stop Audio");
        stopButton.setOnClickListener(
                new View.OnClickListener() {
                    @Override
                    public void onClick(View view) {
                        audioPlayer.setPlaybackEnabled(false);
                    }
                });
        controls.addView(stopButton);

        root.addView(controls);

        status = new TextView(this);
        status.setTextSize(16.0f);
        root.addView(status);

        return root;
    }

    private void updateUi(AudioPlayer.PlayerState state) {
        boolean isPlaying = state == AudioPlayer.PlayerState.STARTED;
        startButton.setEnabled(!isPlaying);
        stopButton.setEnabled(isPlaying);
        status.setText("Current status: " + statusText(state));
    }

    private String statusText(AudioPlayer.PlayerState state) {
        switch (state) {
            case STARTED:
                return "Started";
            case STOPPED:
                return "Stopped";
            case UNKNOWN:
                return "Unknown. Result = " + audioPlayer.getLastResultCode();
            case NO_RESULT_YET:
            default:
                return "No result yet";
        }
    }
}
