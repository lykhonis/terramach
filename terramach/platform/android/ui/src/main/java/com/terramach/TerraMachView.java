/*
 * Terra Mach
 * Copyright [2020] Terra Mach Authors
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>
 */

package com.terramach;

import android.content.Context;
import android.graphics.PixelFormat;
import android.util.AttributeSet;
import android.util.DisplayMetrics;
import android.view.Surface;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import androidx.annotation.Keep;
import androidx.annotation.NonNull;

public final class TerraMachView extends SurfaceView implements SurfaceHolder.Callback {
    private AppThread appThread;

    public TerraMachView(Context context) {
        super(context);
        initialize();
    }

    public TerraMachView(Context context, AttributeSet attrs) {
        super(context, attrs);
        initialize();
    }

    public TerraMachView(Context context, AttributeSet attrs, int defStyleAttr) {
        super(context, attrs, defStyleAttr);
        initialize();
    }

    private void initialize() {
        SurfaceHolder holder = getHolder();
        holder.addCallback(this);
    }

    @SuppressWarnings("unused")
    @Keep
    private float getDevicePixelRatio() {
        return getResources().getDisplayMetrics().density;
    }

    @Override
    public void surfaceCreated(SurfaceHolder holder) {
        appThread = new AppThread();
        appThread.start();
    }

    @Override
    public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {
        appThread.setSurfaceHolder(holder);
    }

    @Override
    public void surfaceDestroyed(SurfaceHolder holder) {
        stopApp();
    }

    private final class AppThread extends Thread {
        private final Object mutex;
        private SurfaceHolder surfaceHolder;

        public AppThread() {
            this.mutex = new Object();
        }

        public void setSurfaceHolder(SurfaceHolder surfaceHolder) {
            synchronized (mutex) {
                this.surfaceHolder = surfaceHolder;
                mutex.notify();
            }
        }

        @Override
        public void run() {
            synchronized (mutex) {
                if (surfaceHolder == null) {
                    try {
                        mutex.wait();
                    } catch (InterruptedException ignore) {
                    }
                    if (surfaceHolder == null) return;
                }
            }
            runApp(surfaceHolder.getSurface());
        }
    }

    @SuppressWarnings("unused")
    @Keep
    private long app;

    private native void runApp(@NonNull Surface surface);

    private native void stopApp();
}
