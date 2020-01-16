/*
 * Terra Mach
 * Copyright [2020] Volodymyr Lykhonis
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

use std::process::Command;
use std::path::{PathBuf, Path};
use std::env;

use crate::{cargo, xcode};

use bindgen;
use bindgen::EnumVariation;

use cc::Build;

pub struct BuildConfiguration {
    source_dir: PathBuf,
    build_dir: PathBuf,
}

impl BuildConfiguration {
    pub fn new(
        source_dir: impl AsRef<Path>,
        build_dir: impl AsRef<Path>,
    ) -> Self {
        BuildConfiguration {
            source_dir: source_dir.as_ref().to_path_buf(),
            build_dir: build_dir.as_ref().to_path_buf(),
        }
    }
}

pub fn build(config: &BuildConfiguration) {
    fetch_dependencies();

    // configure
    if !config.build_dir.join("CMakeCache.txt").exists() {
        assert!(
            Command::new("cmake")
                .current_dir(&config.source_dir)
                .args(&[".", "-B", config.build_dir.to_str().unwrap()])
                .status()
                .expect("Failed to configure Mapbox")
                .success(),
            "Failed while configuring Mapbox",
        );
    }

    // build
    let mut command = Command::new("cmake");
    command.current_dir(&config.source_dir)
        .args(&["--build", config.build_dir.to_str().unwrap()]);

    if let Ok(num_jobs) = env::var("NUM_JOBS") {
        command.arg(format!("-j{}", num_jobs));
    }

    assert!(
        command.status().expect("Failed to build Mapbox").success(),
        "Failed while building Mapbox",
    );

    // link
    match cargo::target().as_strs() {
        (_, "apple", "darwin", _) => {
            cargo::add_link_libs(&[
                "c++",
                "z",
                "sqlite3",
                "framework=Foundation",
            ]);
        }
        _ => {}
    }

    bindgen_gen(config);

    cargo::rerun_if_changed(&config.source_dir);
    cargo::add_link_search(config.build_dir.to_str().unwrap());
    cargo::add_link_libs(&[
        "static=mbgl-core",
        "static=mbgl-vendor-icu",
        "static=mbgl-vendor-parsedate",
        "static=mbgl-vendor-csscolorparser",
        "static=mapbox-bindings",
    ]);
}

fn fetch_dependencies() {
    assert!(
        Command::new("git")
            .current_dir("mapbox-gl-native")
            .args(&["submodule", "update", "--init", "--recursive", "--depth", "1"])
            .status()
            .expect("Failed to fetch Mapbox dependencies")
            .success(),
        "Failed while fetching Mapbox dependencies",
    );
}

fn bindgen_gen(config: &BuildConfiguration) {
    let mut builder = bindgen::Builder::default()
        .raw_line("#![allow(non_upper_case_globals)]")
        .raw_line("#![allow(non_camel_case_types)]")
        .raw_line("#![allow(non_snake_case)]")
        .raw_line("#![allow(dead_code)]")
        .generate_comments(false)
        .layout_tests(true)
        .derive_debug(true)
        .default_enum_style(EnumVariation::Rust {
            non_exhaustive: false,
        })
        .whitelist_function("C_.*")
        .whitelist_type("terramach.*")
        .use_core()
        .clang_arg("-std=c++14")
        .clang_args(&["-x", "c++"])
        .clang_arg("-v");

    let whitelist_types = [
        "mbgl::Map",
    ];

    for whitelist_type in whitelist_types.iter() {
        builder = builder.whitelist_type(whitelist_type);
    }

    let blacklist_types = [
        "std::vector.*",
        "std::set.*",
        "std::unordered_map.*",
        "std::map.*",
        "std::tree.*",
        "std::basic_string.*",
    ];

    for blacklist_type in blacklist_types.iter() {
        builder = builder.blacklist_type(blacklist_type);
    }

    let opaque_types = [
        "std::unique_ptr",
        "std::vector",
        "std::set",
        "std::basic_string",
        "std::experimental::optional",
        "mapbox::feature::feature",
        "mapbox::geometry::geometry",
        "mapbox::geometry::line_string",
        "mapbox::geometry::multi_line_string",
        "mapbox::geometry::polygon",
        "mapbox::geometry::multi_polygon",
        "mapbox::util::variant",
        "mbgl::style::expression::EvaluationResult",
        "mbgl::style::PropertyValue",
    ];

    for opaque_type in opaque_types.iter() {
        builder = builder.opaque_type(opaque_type);
    }

    let aliases = [
        ("mbgl_style_expression_type__Type", "mbgl_style_expression_type_Type"),
        ("mbgl_style_expression_type__Array", "mbgl_style_expression_type_Array"),
        ("mapbox_geometry_box_<T>", "mapbox_geometry_box<T>"),
    ];

    for (new, existing) in aliases.iter() {
        builder = builder.raw_line(format!("pub type {} = {};", new, existing));
    }

    let mut cc_build = Build::new();
    cc_build.cpp(true)
        .out_dir(&config.build_dir)
        .flag("-fno-rtti");

    let headers = [
        "src/backend.h",
        "src/frontend.h",
        "src/map.h",
        "src/scheduler.h",
    ];

    let sources = [
        "src/bindings.cpp",
        "src/backend.cpp",
        "src/frontend.cpp",
        "src/map.cpp",
        "src/scheduler.cpp",
    ];

    for header in headers.iter() {
        cargo::rerun_if_changed(header);
    }

    for source in sources.iter() {
        cc_build.file(source);
        builder = builder.header(*source);
        cargo::rerun_if_changed(source);
    }

    let include_dir = config.source_dir.join("include");
    builder = builder.clang_arg(format!("-I{}", include_dir.display()));
    cc_build.include(include_dir);

    let vendors = [
        Path::new("geometry.hpp").join("include"),
        Path::new("variant").join("include"),
        Path::new("value").join("include"),
        Path::new("weak").join("include"),
        Path::new("typewrapper").join("include"),
        Path::new("optional").to_path_buf(),
        Path::new("geojson.hpp").join("include"),
    ];

    for vendor in vendors.iter() {
        let include_dir = config.source_dir
            .join("vendor")
            .join("mapbox-base")
            .join("mapbox")
            .join(vendor);
        builder = builder.clang_arg(format!("-I{}", include_dir.display()));
        cc_build.include(include_dir);
    }

    let target = cargo::target();
    match target.as_strs() {
        (_, "apple", "darwin", _) => {
            let target = &target.to_string();
            cc_build.target(target);
            if let Some(sdk) = xcode::sdk_path("macosx") {
                builder = builder.clang_arg(format!("-isysroot{}", sdk.to_str().unwrap()));
            } else {
                cargo::warning("failed to get macosx SDK path")
            }
        }
        _ => {}
    }

    if !cfg!(windows) {
        cc_build.flag("-std=c++14");
    }

    cc_build.compile("mapbox-bindings");

    let bindings = builder.generate().expect("Unable to generate bindings");
    let out_path = PathBuf::from("src");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
