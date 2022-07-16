extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn get_dpdk_pkgconfig(arg: &str) -> String {
    let output = Command::new("pkg-config")
        .arg(arg)
        .arg("libdpdk")
        .output()
        .expect("failed to execute pkg-config process");
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn get_dpdk_libs() -> String {
    let libs = get_dpdk_pkgconfig("--libs")
        .split_whitespace()
        .filter_map(|s| {
            if s.starts_with("-L") || s.starts_with("-l") {
                Some(String::from(s))
            } else {
                None
            }
        })
        .collect::<Vec<String>>();
    libs.join(" ")
}

fn main() {
    let rte_target = env::var("RTE_TARGET").ok();
    if rte_target.is_none() {
        panic!("You need to define RTE_TARGET (e.g., x86_64-linux-gnu)");
    }

    let project_path = PathBuf::from(".").canonicalize().unwrap();
    //eprintln!("project path = {}", project_path.to_string_lossy());

    let api_file_path = project_path.join("..").join("udpdk").join("udpdk_api.h");
    //eprintln!("api file = {}", api_file_path.to_string_lossy());

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search=../udpdk/");

    // Tell cargo to tell rustc to link the udpdk library
    println!("cargo:rustc-link-lib=static=udpdk");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed={}", api_file_path.to_string_lossy());

    let pkgconfig_path = project_path
        .join("..")
        .join("deps")
        .join("dpdk")
        .join("install")
        .join("lib")
        .join(rte_target.unwrap())
        .join("pkgconfig");
    //eprintln!("pkgconfig_path = {}", pkgconfig_path.to_string_lossy());
    env::set_var("PKG_CONFIG_PATH", &*pkgconfig_path.to_string_lossy());

    let dpdk_libs = get_dpdk_libs();
    println!("cargo:rustc-flags={}", dpdk_libs);

    //let dpdk_cflags = get_dpdk_pkgconfig("--cflags");
    //eprintln!("dpdk libs = {}", dpdk_libs);
    //eprintln!("dpdk cflags = {}", dpdk_cflags.trim());

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(api_file_path.to_string_lossy())
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
