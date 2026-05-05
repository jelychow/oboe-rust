package com.google.oboe.smoke;

import android.app.Activity;
import android.os.Bundle;
import android.util.Log;
import android.view.View;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.ScrollView;
import android.widget.TextView;
import com.google.oboe.AudioApi;
import com.google.oboe.AudioStream;
import com.google.oboe.AudioStreamBuilder;
import com.google.oboe.PlaybackParameters;

public final class MainActivity extends Activity {
    private static final String TAG = "OboeSmoke";

    private final StringBuilder result = new StringBuilder();
    private TextView output;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        LinearLayout root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setPadding(32, 32, 32, 32);

        Button probeAaudio = createButton("Probe AAudio", new View.OnClickListener() {
            @Override
            public void onClick(View view) {
                smokeApi(AudioApi.AAUDIO);
            }
        });
        Button probeOpenSl = createButton("Probe OpenSL ES", new View.OnClickListener() {
            @Override
            public void onClick(View view) {
                smokeApi(AudioApi.OPENSL_ES);
            }
        });
        Button probeBoth = createButton("Probe Both Backends", new View.OnClickListener() {
            @Override
            public void onClick(View view) {
                smokeApi(AudioApi.AAUDIO);
                smokeApi(AudioApi.OPENSL_ES);
            }
        });

        output = new TextView(this);
        output.setTextSize(16.0f);
        output.setPadding(0, 24, 0, 0);

        ScrollView scrollView = new ScrollView(this);
        scrollView.addView(output);

        root.addView(probeAaudio);
        root.addView(probeOpenSl);
        root.addView(probeBoth);
        root.addView(scrollView, new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                0,
                1.0f));

        setContentView(root);
        runStartupCheck();
    }

    private Button createButton(String text, View.OnClickListener listener) {
        Button button = new Button(this);
        button.setAllCaps(false);
        button.setText(text);
        button.setOnClickListener(listener);
        return button;
    }

    private void runStartupCheck() {
        append("Rust-native Oboe smoke test\n\n");

        try {
            int versionCode = AudioStream.nativeVersionCode();
            append("nativeVersionCode=");
            append(Integer.toString(versionCode));
            append('\n');
            append("JNI library loaded successfully.\n");
            Log.i(TAG, "JNI library loaded successfully, nativeVersionCode=" + versionCode);
        } catch (Throwable error) {
            append("native library load failed: ");
            append(error.toString());
            append('\n');
            Log.e(TAG, "Native library load failed", error);
            return;
        }

        append("\nBackend probes are manual. If a probe fails with AudioTrack -22, ");
        append("the APK and JNI load still succeeded; the current device did not ");
        append("accept that audio backend configuration.\n");
    }

    private void smokeApi(AudioApi api) {
        append('\n');
        append(api.name());
        append(" backend probe\n");
        try (AudioStream stream = new AudioStreamBuilder().setAudioApi(api).openStream()) {
            append("open state=");
            append(Integer.toString(stream.getState()));
            append('\n');
            append("setPlaybackParameters=");
            append(Integer.toString(stream.setPlaybackParameters(PlaybackParameters.defaults())));
            append('\n');
            append("requestStart=");
            append(Integer.toString(stream.requestStart()));
            append('\n');
            append("started state=");
            append(Integer.toString(stream.getState()));
            append('\n');
            append("requestStop=");
            append(Integer.toString(stream.requestStop()));
            append('\n');
            append("stopped state=");
            append(Integer.toString(stream.getState()));
            append('\n');
            Log.i(TAG, api.name() + " backend probe succeeded");
        } catch (Throwable error) {
            append("backend unavailable: ");
            append(error.getClass().getSimpleName());
            if (error.getMessage() != null) {
                append(": ");
                append(error.getMessage());
            }
            append('\n');
            Log.w(TAG, api.name() + " backend probe failed", error);
        }
    }

    private void append(String text) {
        result.append(text);
        output.setText(result.toString());
    }

    private void append(char value) {
        result.append(value);
        output.setText(result.toString());
    }
}
