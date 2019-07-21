extern crate bindgen;

use std::env;
use std::path::{PathBuf, Path};
use std::subprocess::Command;

fn build_lightning(ldir: Path, prefix: Path) {
    Command::new("git")
        .arg("clone")
        .arg("--recursive")
        .arg("git://git.ghostscript.com/mupdf.git")
        .spawn()
        .expect("Failed to clone MuPDF")
        .wait()
        .expect("Failed to wait for git process");
}

fn main() {

    println!("cargo:rustc-link-lib=lightning");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
