extern crate bindgen;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn build_lightning(prefix: &str) {
    Command::new("./build-lightning.sh")
        .arg(prefix)
        .output().unwrap();
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let prefix = out_path.join("lightning-prefix");
    let libdir = prefix.join("lib");
    let incdir = prefix.join("include");
    let header = incdir.join("lightning.h");

    build_lightning(out_path.join("lightning-prefix").to_str().unwrap());

    println!("cargo:rustc-link-search=native={}", libdir.to_str().unwrap());
    println!("cargo:rustc-link-lib=static=lightning");

    let bindings = bindgen::Builder::default()
        .header(header.to_str().unwrap())
        .clang_arg(format!("-I{}", incdir.to_str().unwrap()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
