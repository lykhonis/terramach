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

pub use ::egl::*;

pub fn android_choose_config(display: EGLDisplay) -> EGLConfig {
    choose_config(
        display,
        &[
            EGL_SURFACE_TYPE, EGL_PBUFFER_BIT,
            EGL_RENDERABLE_TYPE, EGL_OPENGL_ES2_BIT,
            EGL_RED_SIZE, 8,
            EGL_GREEN_SIZE, 8,
            EGL_BLUE_SIZE, 8,
            EGL_ALPHA_SIZE, 8,
            EGL_NONE,
        ],
        1,
    ).expect("Unable to find appropriate EGL configuration")
}

pub fn android_new_context(display: EGLDisplay, config: EGLConfig) -> EGLContext {
    create_context(
        display,
        config,
        EGL_NO_CONTEXT,
        &[
            EGL_CONTEXT_CLIENT_VERSION, 2,
            EGL_NONE,
        ],
    ).expect("Unable to create a EGL context")
}
