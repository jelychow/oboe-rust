package com.google.oboe.remoteaar;

import android.app.Activity;
import android.os.Bundle;
import android.view.Gravity;
import android.widget.LinearLayout;
import android.widget.TextView;
import com.google.oboe.AudioStream;

public final class MainActivity extends Activity {
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        TextView title = new TextView(this);
        title.setText("Oboe remote AAR demo");
        title.setTextSize(22f);

        TextView status = new TextView(this);
        status.setText("Loaded remote oboe-rust-android AAR. Native version: "
                + AudioStream.nativeVersionCode());
        status.setTextSize(16f);

        LinearLayout layout = new LinearLayout(this);
        layout.setOrientation(LinearLayout.VERTICAL);
        layout.setGravity(Gravity.CENTER);
        int padding = 48;
        layout.setPadding(padding, padding, padding, padding);
        layout.addView(title);
        layout.addView(status);

        setContentView(layout);
    }
}
