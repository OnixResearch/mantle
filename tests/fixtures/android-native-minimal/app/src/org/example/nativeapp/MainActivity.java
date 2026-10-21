package org.example.nativeapp;

import android.app.Activity;
import android.os.Bundle;
import android.widget.TextView;

public final class MainActivity extends Activity {
    static { System.loadLibrary("mantle_android_native_minimal"); }
    private native int value();

    @Override public void onCreate(Bundle state) {
        super.onCreate(state);
        TextView view = new TextView(this);
        view.setText("JNI value: " + value());
        setContentView(view);
    }
}
