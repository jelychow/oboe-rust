package com.google.oboe.samples;

import android.app.Activity;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.view.View;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.RadioButton;
import android.widget.RadioGroup;
import android.widget.ScrollView;
import android.widget.TextView;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

public final class MainActivity extends Activity {
    private static final int SAMPLE_DURATION_MILLIS = 850;

    private static final String[] SAMPLE_NAMES = {
        "hello-oboe",
        "minimaloboe",
        "LiveEffect",
        "MegaDrone",
        "SoundBoard",
        "audio-device",
        "drumthumper",
        "powerplay",
        "RhythmGame",
        "iolib",
        "parselib",
        "shared",
        "debug-utils"
    };

    private final ExecutorService executor = Executors.newSingleThreadExecutor();
    private final Handler mainHandler = new Handler(Looper.getMainLooper());
    private final StringBuilder log = new StringBuilder();

    private TextView output;
    private RadioGroup apiGroup;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(createContentView());
        append("Rust Oboe samples\n");
        append("nativeSampleCount=" + RustSampleRunner.nativeSampleCount() + "\n");
    }

    @Override
    protected void onDestroy() {
        executor.shutdownNow();
        super.onDestroy();
    }

    private View createContentView() {
        LinearLayout root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setPadding(32, 32, 32, 32);

        TextView title = new TextView(this);
        title.setText("Rust Oboe Samples");
        title.setTextSize(22.0f);
        root.addView(title);

        apiGroup = new RadioGroup(this);
        apiGroup.setOrientation(RadioGroup.HORIZONTAL);
        RadioButton aaudio = new RadioButton(this);
        aaudio.setId(RustSampleRunner.API_AAUDIO);
        aaudio.setText("AAudio");
        aaudio.setChecked(true);
        RadioButton opensl = new RadioButton(this);
        opensl.setId(RustSampleRunner.API_OPENSL_ES);
        opensl.setText("OpenSL ES");
        apiGroup.addView(aaudio);
        apiGroup.addView(opensl);
        root.addView(apiGroup);

        Button runAll = new Button(this);
        runAll.setAllCaps(false);
        runAll.setText("Run all samples");
        runAll.setOnClickListener(
                new View.OnClickListener() {
                    @Override
                    public void onClick(View view) {
                        runAllSamples();
                    }
                });
        root.addView(runAll);

        for (int index = 0; index < SAMPLE_NAMES.length; index++) {
            final int sampleId = index;
            Button button = new Button(this);
            button.setAllCaps(false);
            button.setText("Run " + SAMPLE_NAMES[index]);
            button.setOnClickListener(
                    new View.OnClickListener() {
                        @Override
                        public void onClick(View view) {
                            runSample(sampleId);
                        }
                    });
            root.addView(button);
        }

        output = new TextView(this);
        output.setTextSize(14.0f);
        output.setPadding(0, 24, 0, 0);
        ScrollView scrollView = new ScrollView(this);
        scrollView.addView(output);
        root.addView(
                scrollView,
                new LinearLayout.LayoutParams(
                        LinearLayout.LayoutParams.MATCH_PARENT, 0, 1.0f));

        return root;
    }

    private void runAllSamples() {
        final int audioApi = selectedAudioApi();
        executor.execute(
                new Runnable() {
                    @Override
                    public void run() {
                        for (int sampleId = 0; sampleId < SAMPLE_NAMES.length; sampleId++) {
                            runSampleOnCurrentThread(sampleId, audioApi);
                        }
                    }
                });
    }

    private void runSample(final int sampleId) {
        final int audioApi = selectedAudioApi();
        executor.execute(
                new Runnable() {
                    @Override
                    public void run() {
                        runSampleOnCurrentThread(sampleId, audioApi);
                    }
                });
    }

    private void runSampleOnCurrentThread(int sampleId, int audioApi) {
        postAppend("Running " + SAMPLE_NAMES[sampleId] + " on " + apiName(audioApi) + "...\n");
        int written =
                RustSampleRunner.nativeRunSample(sampleId, audioApi, SAMPLE_DURATION_MILLIS);
        if (written >= 0) {
            postAppend("OK " + SAMPLE_NAMES[sampleId] + ": wrote " + written + " samples\n");
        } else {
            postAppend("FAILED " + SAMPLE_NAMES[sampleId] + "\n");
        }
    }

    private int selectedAudioApi() {
        int checkedId = apiGroup.getCheckedRadioButtonId();
        if (checkedId == RustSampleRunner.API_OPENSL_ES) {
            return RustSampleRunner.API_OPENSL_ES;
        }
        return RustSampleRunner.API_AAUDIO;
    }

    private static String apiName(int audioApi) {
        return audioApi == RustSampleRunner.API_OPENSL_ES ? "OpenSL ES" : "AAudio";
    }

    private void postAppend(final String text) {
        mainHandler.post(
                new Runnable() {
                    @Override
                    public void run() {
                        append(text);
                    }
                });
    }

    private void append(String text) {
        log.append(text);
        output.setText(log.toString());
    }
}
