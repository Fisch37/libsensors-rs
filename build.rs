use std::{env, path::PathBuf};

use bindgen::callbacks::{IntKind, ParseCallbacks};
use regex::Regex;

/// This code has been shamelessly adapted from
/// https://rust-lang.github.io/rust-bindgen/tutorial-3.html

// well okay, I may have made some slight modifications
#[derive(Debug)]
struct SetMacroTypeFromRegex(Regex, IntKind);
impl ParseCallbacks for SetMacroTypeFromRegex {
    fn int_macro(&self, _name: &str, _value: i64) -> Option<bindgen::callbacks::IntKind> {
        Some(self.1).filter(|_| self.0.is_match(_name))
    }
}

fn main() {
    // println!("cargo:rustc-link-search")
    println!("cargo:rustc-link-lib=sensors");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(SetMacroTypeFromRegex(
            Regex::new(r"^SENSORS_BUS_(TYPE|NR)_.*$").expect("Failed to parse macro"),
            IntKind::Short
        )))
        .constified_enum_module(r"sensors_(feature_type|subfeature_type)")
        .generate()
        .expect("unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}