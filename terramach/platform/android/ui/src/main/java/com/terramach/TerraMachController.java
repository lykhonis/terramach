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
import androidx.annotation.Keep;
import androidx.annotation.NonNull;

public final class TerraMachController {
    private final Context context;
    private TerraMachView view;

    public TerraMachController(@NonNull Context context) {
        this.context = context;
        create();
    }

    public void onDestroy() {
        view = null;
        destroy();
    }

    public void onStart() {
    }

    public void onStop() {
    }

    @NonNull
    public TerraMachView getView() {
        if (view == null) {
            view = new TerraMachView(context);
        }
        return view;
    }

    private native void create();

    private native void destroy();

    private native void start();

    private native void stop();

    @SuppressWarnings("unused")
    @Keep
    private long eventEmitter;

    static {
        System.initialize();
    }
}
