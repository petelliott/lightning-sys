extern crate bindgen;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn build_lightning(prefix: &str) {
    Command::new("./build-lightning.sh")
        .arg(prefix)
        .output().unwrap();
}

/*
TODO
fn has_lightning() -> bool {
    let out = Command::new("bash")
        .arg("-c")
        .arg("ldconfig -p | grep liblightning")
        .output().unwrap();

    match out.status.code() {
        None => panic!("process terminated by signal"),
        Some(code) =>
            match code {
                0 => true,
                1 => false,
                _ => panic!("unexpected exit code"),
            },
    }
}
*/

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let prefix = out_path.join("lightning-prefix");
    let libdir = prefix.join("lib");
    let incdir = prefix.join("include");

    build_lightning(prefix.to_str().unwrap());

    println!("cargo:rustc-link-search=native={}", libdir.to_str().unwrap());

    println!("cargo:rustc-link-lib=static=lightning");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", incdir.to_str().unwrap()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
