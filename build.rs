extern crate attohttpc;
extern crate bindgen;
extern crate flate2;
extern crate tar;

use flate2::read::GzDecoder;
use std::env;
use std::io::Read;
use std::path::{PathBuf, Path};
use std::process::Command;
use tar::Archive;

fn build_lightning(prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    let release = include_str!("release");
    let target = format!("http://ftp.gnu.org/gnu/lightning/{}.tar.gz", release);
    unpack(attohttpc::get(&target).send()?.split().2, prefix)?;

    let cflags = cc::Build::new().get_compiler().cflags_env();
    let flags = vec![
            ("CFLAGS", cflags.clone()),
            ("LDFLAGS", cflags.clone()),
        ];

    let run =
        Command::new("./build-lightning.sh")
            .envs(flags)
            .arg(prefix)
            .arg(release)
            .status();

    match run {
        Ok(x) if x.success() => Ok(()),
        _ => Err(format!("failed to build {}", release).into()),
    }
}

fn lightning_built(prefix: &Path) -> bool {
    // Since a cross-platform name for the actual library file is hard to
    // compute, just look for the "lib" directory, which implies that the
    // `install` target succeeded.
    prefix.join("lib").exists()
}

fn unpack<P: AsRef<Path>>(tgz: impl Read, outdir: P) -> Result<(), std::io::Error> {
    let tar = GzDecoder::new(tgz);
    let mut archive = Archive::new(tar);
    archive.unpack(outdir)?;

    Ok(())
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let prefix = out_path.join("lightning-prefix");
    let libdir = prefix.join("lib");
    let incdir = prefix.join("include");

    if !lightning_built(&prefix) {
        build_lightning(prefix.to_str().unwrap()).unwrap();
    }
    cc::Build::new()
        .include(incdir.clone())
        .file("C/register.c")
        .compile("lightningsys");

    println!("cargo:rustc-link-search=native={}", libdir.to_str().unwrap());

    println!("cargo:rustc-link-lib=static=lightning");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        // Tell bindgen to regenerate bindings if the wrapper.h's contents or transitively
        // included files change.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .whitelist_function(".+_jit")
        .whitelist_function("_?jit_.*")
        .whitelist_type("_?jit_.*")
        .whitelist_var("_?jit_.*")
        .whitelist_function("lgsys_.*")
        .whitelist_var("lgsys_.*")
        .rustfmt_bindings(true)
        .clang_arg(format!("-I{}", incdir.to_str().unwrap()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
