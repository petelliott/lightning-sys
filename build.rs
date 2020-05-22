extern crate attohttpc;
extern crate bindgen;
extern crate flate2;
extern crate tar;

use flate2::read::GzDecoder;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::io::{Read, Write};
use std::panic::AssertUnwindSafe;
use std::path::{PathBuf, Path};
use std::process::Command;
use std::rc::Rc;
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

// We need the interior mutability of `RefCell` to work around the fact that the
// `func_macro` callback is not called with a `mut` reference. We need
// `AssertUnwindSafe` to allow `RefCell` to be used within a `catch_unwind`
// context inside `bindgen`. We use `Rc` to so that we can get back the `T`
// after `bindgen` is done with the `Callbacks`, by keeping the reference alive
// without having to make it `&'static` instead.
type HideUnwinding<T> = Rc<AssertUnwindSafe<RefCell<T>>>;

struct Callbacks {
    wrapped: bindgen::CargoCallbacks,
    state: HideUnwinding<BTreeMap<String, Vec<u8>>>,
}

impl std::fmt::Debug for Callbacks {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        fmt.debug_struct("Callbacks")
            .field("wrapped", &self.wrapped)
            .field("state", &self.state.borrow())
            .finish()
    }
}

impl Callbacks {
    fn new(bt: HideUnwinding<BTreeMap<String, Vec<u8>>>) -> Self {
        let wrapped = bindgen::CargoCallbacks;
        let state = Rc::clone(&bt);
        Self { wrapped, state }
    }
}

impl bindgen::callbacks::ParseCallbacks for Callbacks {
    fn func_macro(&self, name: &str, value: &[&[u8]]) {
        self.state.borrow_mut().insert(name.to_owned(), value.join(" ".as_bytes()));
    }

    fn include_file(&self, filename: &str) {
        self.wrapped.include_file(filename)
    }
}

fn generate(pairs: impl IntoIterator<Item=(String,String)>)
    -> impl IntoIterator<Item=String>
{
    let pairs: BTreeMap<_,_> = pairs.into_iter().collect();

    let suffixes = vec![
        "_f", "_d",
        "_u",
        "_c", "_i", "_l", "_s",
        "_uc", "_ui", "_ul", "_us",
        "_dp", "_fp", "_p", "_pw", "_pwd", "_pwf", "_pww", "_qww",
        "_w", "_wd", "_wf", "_wp", "_ww", "_wwd", "_wwf", "_www",
    ];

    // This closure is not particularly efficient, and might be reworked.
    let chop_suffixes = |orig: &str| {
        let mut pieces = Vec::new();

        let mut start = orig;
        loop {
            let mut truncated = start;
            for suff in &suffixes {
                if truncated.ends_with(suff) {
                    let idx = truncated.len() - suff.len();
                    pieces.push((*suff).to_owned());
                    truncated = &truncated[..idx];
                }
            }

            if truncated.len() == start.len() {
                pieces.push(truncated.to_string());
                pieces.reverse();
                break pieces;
            }
            start = truncated;
        }
    };

    let stems: BTreeSet<_> =
        pairs
            .keys()
            .map(|e| e.split('(').next().unwrap())
            .map(|e| e.trim_start_matches("jit_"))
            .map(chop_suffixes)
            .collect();

    let roots: BTreeSet<_> =
        stems.iter()
            .map(|e| e[0].clone())
            .map(|e|
                 match e.as_bytes() {
                     [r @ .., b'i'] | [r @ .., b'r'] => r.to_owned(),
                     _ => e.into_bytes(),
                 })
            .map(String::from_utf8)
            .map(Result::unwrap)
            .collect()
            ;

    let kind_match = |needle: &str, haystack: &str| {
        let last_char = haystack.chars().last().unwrap();
        haystack.starts_with(needle)
            && (haystack.len() - needle.len() < 2)
            && (haystack.len() == needle.len() || last_char == 'r' || last_char == 'i')
    };
    let variants: BTreeMap<_,Vec<_>> =
        roots.iter()
            .map(|r| (r,stems.iter().filter(|s| kind_match(r, &s[0])).collect()))
            .collect();

    let inverse_variants: BTreeMap<String,String> =
        variants.iter().fold(BTreeMap::new(), |mut iv, (k, v)| {
            iv.extend(v.iter().map(|x| (x.concat(), (*k).to_string())));
            iv
        });

    let collected: Vec<_> =
        pairs.keys()
            .map(|k| {
                let brief = &k[..k.find('(').unwrap()];
                let core = brief.trim_start_matches("jit_");
                let iv = &inverse_variants[core];
                let outs =
                    std::iter::once(iv.clone())
                        .chain(
                            variants[&iv].iter()
                                .find(|e| core == e.concat())
                                .unwrap()
                                .iter()
                                .enumerate()
                                .filter_map(|(idx, v)|
                                     if idx == 0 {
                                         let v = v.trim_start_matches(iv).to_string();
                                         if v.is_empty() { None } else { Some(v) }
                                     } else {
                                         Some(v.clone())
                                     }
                                )
                        )
                        .collect::<Vec<_>>()
                        .join(", ")
                        ;
                let orig = pairs[k].clone();
                (k, outs, orig, core)
            })
            .collect();

        collected
            .iter()
            .map(|(k, pieces, orig, core)|
                 format!(
                    "jit_entry!{{ {k:w_k$} => {core:w_core$} => [ {pieces:w_pieces$} ] => {orig:w_orig$} }}",
                    k     =k     , w_k     =collected.iter().map(|x| x.0.len()).max().unwrap_or(0),
                    pieces=pieces, w_pieces=collected.iter().map(|x| x.1.len()).max().unwrap_or(0),
                    orig  =orig  , w_orig  =collected.iter().map(|x| x.2.len()).max().unwrap_or(0),
                    core  =core  , w_core  =collected.iter().map(|x| x.3.len()).max().unwrap_or(0),
            ))
            .collect::<Vec<_>>()
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

    let bt = BTreeMap::new();
    let ce = AssertUnwindSafe(RefCell::new(bt));
    let rc = Rc::new(ce);
    let cb = Callbacks::new(Rc::clone(&rc));

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        // Tell bindgen to regenerate bindings if the wrapper.h's contents or transitively
        // included files change.
        .parse_callbacks(Box::new(cb))
        .whitelist_function(".+_jit")
        .whitelist_function("_?jit_.*")
        .whitelist_type("_?jit_.*")
        .whitelist_var("_?jit_.*")
        .whitelist_function("lgsys_.*")
        .whitelist_var("lgsys_.*")
        .rustified_non_exhaustive_enum("jit_code_t")
        .rustfmt_bindings(true)
        .clang_arg(format!("-I{}", incdir.to_str().unwrap()))
        .generate()
        .expect("Unable to generate bindings");

    let rc = rc.borrow();
    let relevant =
        rc
            .iter()
            .filter(|(key, _)| key.starts_with("jit_"))
            .map(|(key, value)| {
                (key.to_owned(), String::from_utf8(value.to_owned()).unwrap())
            })
            .filter(|(_, value)| value.contains("jit_new_node"));

    let output = generate(relevant);
    let mut file = std::fs::File::create(out_path.join("entries.rs")).unwrap();
    writeln!(file, "jit_entries!{{").unwrap();
    for line in output {
        writeln!(file, "    {}", line).unwrap();
    }
    writeln!(file, "}}").unwrap();

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
