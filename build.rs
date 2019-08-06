extern crate bindgen;

use std::env;
use std::path::{PathBuf, Path};
use std::process::Command;
use std::fs;
use std::io::Result;

fn build_lightning(prefix: &str) {
    Command::new("./build-lightning.sh")
        .arg(prefix)
        .output().unwrap();
}

fn build_c(prefix: &str) {
    Command::new("make")
        .env("PREFIX", prefix)
        .arg("-C")
        .arg("C/")
        .output().unwrap();
}

fn lightning_built(prefix: &Path) -> bool {
    prefix.exists()
}

fn _need_bindings_built_res(prefix: &PathBuf) -> Result<bool> {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mt1 = fs::metadata(prefix.join("include").join("lightning.h").to_str().unwrap())?.modified()?;
    let mt2 = fs::metadata("C/lightning-sys.h")?.modified()?;

    let targett = fs::metadata(out_path.join("bindings.rs").to_str().unwrap())?.modified()?;

    Ok(targett < mt1 || targett < mt2)
}

fn need_bindings_built(prefix: &PathBuf) -> bool {
    match _need_bindings_built_res(prefix) {
        Ok(b) => b,
        Err(_) => true,
    }
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let prefix = out_path.join("lightning-prefix");
    let libdir = prefix.join("lib");
    let incdir = prefix.join("include");

    if !lightning_built(&prefix) {
        build_lightning(prefix.to_str().unwrap());
    }
    build_c(prefix.to_str().unwrap());

    println!("cargo:rustc-link-search=native={}", libdir.to_str().unwrap());

    println!("cargo:rustc-link-lib=static=lightning");
    println!("cargo:rustc-link-lib=static=lightningsys");

    if need_bindings_built(&prefix) {
        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            .rustfmt_bindings(true)
            .clang_arg(format!("-I{}", incdir.to_str().unwrap()))
            .generate()
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}
