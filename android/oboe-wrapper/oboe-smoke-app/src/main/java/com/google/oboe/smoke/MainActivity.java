package com.google.oboe.smoke;

import android.app.Activity;
import android.os.Bundle;
import android.widget.TextView;
import com.google.oboe.AudioApi;
import com.google.oboe.AudioStream;
import com.google.oboe.AudioStreamBuilder;
import com.google.oboe.PlaybackParameters;

public final class MainActivity extends Activity {
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        TextView output = new TextView(this);
        output.setTextSize(16.0f);
        output.setPadding(32, 32, 32, 32);
        output.setText(runSmokeTest());
        setContentView(output);
    }

    private static String runSmokeTest() {
        StringBuilder result = new StringBuilder();
        result.append("Rust-native Oboe smoke test\n\n");

        try {
            result.append("nativeVersionCode=").append(AudioStream.nativeVersionCode()).append('\n');
        } catch (Throwable error) {
            result.append("native library load failed: ").append(error).append('\n');
            return result.toString();
        }

        smokeApi(result, AudioApi.AAUDIO);
        smokeApi(result, AudioApi.OPENSL_ES);
        return result.toString();
    }

    private static void smokeApi(StringBuilder result, AudioApi api) {
        result.append('\n').append(api.name()).append('\n');
        try (AudioStream stream = new AudioStreamBuilder().setAudioApi(api).openStream()) {
            result.append("open state=").append(stream.getState()).append('\n');
            result.append("setPlaybackParameters=")
                    .append(stream.setPlaybackParameters(PlaybackParameters.defaults()))
                    .append('\n');
            result.append("requestStart=").append(stream.requestStart()).append('\n');
            result.append("started state=").append(stream.getState()).append('\n');
            result.append("requestStop=").append(stream.requestStop()).append('\n');
            result.append("stopped state=").append(stream.getState()).append('\n');
        } catch (Throwable error) {
            result.append("failed: ").append(error.getClass().getSimpleName());
            if (error.getMessage() != null) {
                result.append(": ").append(error.getMessage());
            }
            result.append('\n');
        }
    }
}
