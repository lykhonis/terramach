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

use std::env;
use std::path::Path;

use crate::cargo;

use bindgen;
use bindgen::EnumVariation;

pub fn build() {
    generate_bindings();
}

fn generate_bindings() {
    let ndk_dir = env::var("ANDROID_NDK").expect("ANDROID_NDK variable is not set");
    let ndk_dir = Path::new(&ndk_dir);
    let sysroot_dir = ndk_dir.join("sysroot");
    let include_dir = sysroot_dir.join("usr").join("include");
    let target = cargo::target().to_triplet();

    let mut builder = bindgen::Builder::default()
        .raw_line("#![allow(non_upper_case_globals)]")
        .raw_line("#![allow(non_camel_case_types)]")
        .raw_line("#![allow(non_snake_case)]")
        .raw_line("#![allow(dead_code)]")
        .generate_comments(false)
        .layout_tests(true)
        .derive_debug(true)
        .default_enum_style(EnumVariation::Consts)
        .use_core()
        .clang_arg("-std=c++14")
        .clang_args(&["-x", "c++"])
        .clang_arg("-v")
        .clang_arg(format!("--target={}", target))
        .clang_arg(format!("-isysroot={}", sysroot_dir.display()))
        .clang_arg(format!("-I{}", include_dir.display()))
        .clang_arg(format!("-I{}", include_dir.join(target).display()));

    let headers = [
        "android/looper.h",
        "android/choreographer.h",
        "android/log.h",
        "android/native_window_jni.h",
        "android/native_window.h",
        "sys/timerfd.h",
        "sys/unistd.h",
    ];
    for header in &headers {
        builder = builder.header(include_dir.join(header).display().to_string());
    }

    let bindings = builder.generate().expect("Failed to generate Android NDK bindings");
    let out_path = cargo::crate_directory()
        .join("ui")
        .join("platform")
        .join("android")
        .join("bindings.rs");
    bindings.write_to_file(out_path).expect("Failed to write Android bindings");

    cargo::rerun_if_env_changed("ANDROID_NDK");
}
