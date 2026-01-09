use std::{env, path::PathBuf};

/// This code has been shamelessly adapted from
/// https://rust-lang.github.io/rust-bindgen/tutorial-3.html

fn main() {
    // println!("cargo:rustc-link-search")
    println!("cargo:rustc-link-lib=sensors");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}