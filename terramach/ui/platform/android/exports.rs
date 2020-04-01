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

#[macro_export]
macro_rules! export_jni {
    (fn $name:ident($( $arg:ident : $type:ty ),*) -> $ret:ty) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        pub extern "C" fn $name($( $arg : $type),*) -> $ret {
            $crate::platform::$name($( $arg ),*)
        }
    };
    (fn $name:ident($( $arg:ident : $type:ty ),*)) => {
        $crate::export_jni!(fn $name($( $arg : $type),*) -> ());
    }
}

#[macro_export]
macro_rules! export_functions {
    () => {
        use $crate::platform::jni::JNIEnv;
        use $crate::platform::jni::objects::{JClass, JObject};
        use $crate::platform::jni::sys::jfloat;

        $crate::export_jni!(fn Java_com_terramach_System_initialize(env: JNIEnv<'static>, class: JClass));

        $crate::export_jni!(fn Java_com_terramach_TerraMachView_runApp(env: JNIEnv<'static>, obj: JObject, surface: JObject));
        $crate::export_jni!(fn Java_com_terramach_TerraMachView_stopApp(env: JNIEnv<'static>, obj: JObject));

        $crate::export_jni!(fn Java_com_terramach_TerraMachController_create(env: JNIEnv<'static>, obj: JObject));
        $crate::export_jni!(fn Java_com_terramach_TerraMachController_destroy(env: JNIEnv<'static>, obj: JObject));
        $crate::export_jni!(fn Java_com_terramach_TerraMachController_start(env: JNIEnv<'static>, obj: JObject));
        $crate::export_jni!(fn Java_com_terramach_TerraMachController_stop(env: JNIEnv<'static>, obj: JObject));
    }
}
