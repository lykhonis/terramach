package com.terramach.counter;

import androidx.appcompat.app.AppCompatActivity;
import android.os.Bundle;

import com.terramach.TerraMachController;

public class MainActivity extends AppCompatActivity {
    private TerraMachController controller;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        controller = new TerraMachController(this);
        setContentView(controller.getView());
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        controller.onDestroy();
    }

    @Override
    protected void onStart() {
        super.onStart();
        controller.onStart();
    }

    @Override
    protected void onStop() {
        super.onStop();
        controller.onStop();
    }
}
