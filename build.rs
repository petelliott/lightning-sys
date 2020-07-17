extern crate attohttpc;
extern crate bindgen;
extern crate flate2;
extern crate tar;

use flate2::read::GzDecoder;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::env;
use std::io::{Read, Write};
use std::ops::Index;
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
            ("LDFLAGS", cflags),
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
        self.state.borrow_mut().insert(name.to_owned(), value.join(b" " as &[u8]));
    }

    fn include_file(&self, filename: &str) {
        self.wrapped.include_file(filename)
    }
}

/// Separates a string into its constituent parts by removing known suffixes
/// from the end of the input string.
fn chop_suffixes(orig: &str) -> Vec<&str> {
    const SPECIALS: &[&str] = &["new_node", "va_arg", "extr", "truncr"];
    for &special in SPECIALS {
        if orig.starts_with(special) {
            if orig == special {
                return vec![special];
            } else {
                return vec![special, &orig[special.len()..]];
            }
        }
    }

    let num_underscores = orig.matches('_').count();

    // Handle special internal movs
    if orig.starts_with("mov") && num_underscores > 1 {
        // Accommodate the length of the initial "movr" or "movi".
        // Treat multiple suffixes as one concatenated suffix in this case.
        return vec![&orig[..4], &orig[4..]];
    }

    if num_underscores == 0 {
        return vec![orig];
    }

    assert_eq!(num_underscores, 1);

    const SUFFIXES: &[&str] = &[
        "_f", "_d",
        "_u",
        "_c", "_i", "_l", "_s",
        "_uc", "_ui", "_ul", "_us",
    ];

    for &suff in SUFFIXES {
        if orig.ends_with(suff) {
            let (a, b) = orig.split_at(orig.len() - suff.len());
            return vec![a, b];
        }
    }

    // Handle anomalies like va_end
    vec![orig]
}

struct Record {
    entry: String,
    stem: String,
    pieces: Vec<String>,
    orig: String,
}

type Pieces<'a> = Vec<&'a Vec<&'a str>>;
type VariantMap<'a> = BTreeMap<&'a str,Pieces<'a>>;
type InverseVariantMap<'a> = BTreeMap<String,&'a str>;

fn extract<'a>(
    variants: &impl Index<&'a str,Output=Pieces<'a>>,
    inverse_variants: &impl Index<&'a str,Output=&'a str>,
    entry: &'a str,
    orig: &'a str,
) -> Record {
    let brief = &entry[..entry.find('(').unwrap()];
    let core = brief.trim_start_matches("jit_");
    let iv = &inverse_variants[core];
    let pieces =
        std::iter::once(iv.clone())
            .chain(
                variants[iv].iter()
                    .find(|e| core == e.concat())
                    .unwrap()
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, v)|
                         if idx == 0 {
                             let v = v.trim_start_matches(iv);
                             if v.is_empty() { None } else { Some(v) }
                         } else {
                             Some(v)
                         }
                    )
            )
            .map(ToString::to_string)
            .collect::<Vec<_>>();
    let stem = core.to_string();
    let entry = entry.to_string();
    let orig = orig.to_string();
    Record { entry, stem, pieces, orig }
}

/// Takes a list of macro left-hand-sides (like `["jit_stxr_i(u,v,w)"]`) and
/// produces a deduplicated list of parsed pieces (like `[["stxr", "_i"]]`).
fn make_stems<'a>(keys: impl Iterator<Item=&'a &'a str>) -> Vec<Vec<&'a str>> {
    let mut out: Vec<_> =
        keys
            .map(|e| e.split('(').next().unwrap())
            .map(|e| e.trim_start_matches("jit_"))
            .map(chop_suffixes)
            .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// Takes a list like that produced by `make_stems` and produces a deduplicated
/// list of cores (for example, `"stxr"` becomes `["stx", "r"]`).
fn make_roots<'a>(stems: impl Iterator<Item=&'a Vec<&'a str>>) -> Vec<&'a str> {
    let mut out: Vec<_> =
        stems
            .map(|e|
                 if e[0].ends_with(|c| c == 'r' || c == 'i') {
                     &e[0][..e[0].len()-1]
                 } else {
                     e[0]
                 }
            )
            .collect();
    out.sort_unstable();
    out.dedup();
    out
}

fn make_variant_maps<'a>(
    roots: impl Iterator<Item=&'a str>,
    stems: &'a [Vec<&'a str>],
) -> (VariantMap<'a>, InverseVariantMap<'a>) {
    let kind_match = |needle: &str, haystack: &str| {
        let last_char = haystack.chars().last().unwrap();
        haystack.starts_with(needle)
            && (haystack.len() - needle.len() < 2)
            && (haystack.len() == needle.len() || last_char == 'r' || last_char == 'i')
    };
    let variants: BTreeMap<_,Vec<_>> =
        roots
            .map(|r| (r,stems.iter().filter(|s| kind_match(r, &s[0])).collect()))
            .collect();

    let inverse_variants =
        variants.iter().fold(BTreeMap::new(), |mut iv, (&k, v)| {
            iv.extend(v.iter().map(|x| (x.concat(), k)));
            iv
        });

    (variants, inverse_variants)
}

/// Parses a list of macro definitions into a list of `Record`s.
fn parse_macros<'a>(pairs: &[(&'a str,&'a str)]) -> Vec<Record> {
    let stems: Vec<_> = make_stems(pairs.iter().map(|(key, _value)| key)).into_iter().collect();

    let roots = make_roots(stems.iter()).into_iter();

    let (variants, inverse_variants) = make_variant_maps(roots, stems.as_slice());

    pairs
        .iter()
        .map(|(k, v)| extract(&variants, &inverse_variants, k, &v))
        .collect()
}

/// Takes a `Vec` of `Record`s and generates `jit_entry!{}` macro invocations
/// for them, with pretty-printing.
fn make_printable(collected: Vec<Record>) -> Vec<String> {
    let strings: Vec<_> =
        collected
            .into_iter()
            .map(|Record { entry, stem, pieces, orig }| {
                let pieces = pieces.join(", ");
                (entry, stem, pieces, orig)
            })
            .collect();

    type Sizer = dyn Fn(&(String, String, String, String)) -> usize;
    let get_width = |closure: &Sizer|
        strings.iter().map(closure).max().unwrap_or(0);

    strings
        .iter()
        .map(|(entry, stem, pieces, orig)|
             format!(
                "jit_entry!{{ {entry:w_entry$} => {stem:w_stem$} => [ {pieces:w_pieces$} ] => {orig:w_orig$} }}",
                entry =entry , w_entry =get_width(&|x| x.0.len()),
                stem  =stem  , w_stem  =get_width(&|x| x.1.len()),
                pieces=pieces, w_pieces=get_width(&|x| x.2.len()),
                orig  =orig  , w_orig  =get_width(&|x| x.3.len()),
            )
        )
        .collect()
}

fn main() -> std::io::Result<()> {
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
        .header(incdir.join("lightning.h").to_str().unwrap())
        .header("C/lightning-sys.h")
        // Tell bindgen to regenerate bindings if the wrapper.h's contents or transitively
        // included files change.
        .parse_callbacks(Box::new(cb))
        .whitelist_function(".+_jit")
        .whitelist_function("_?jit_.*")
        .whitelist_type("_?jit_.*")
        .whitelist_var("_?jit_.*")
        .whitelist_function("lgsys_.*")
        .whitelist_var("lgsys_.*")
        .rust_target(bindgen::RustTarget::Stable_1_36)
        .rustified_enum("jit_code_t")
        .rustfmt_bindings(true)
        .clang_arg(format!("-I{}", incdir.to_str().unwrap()))
        .generate()
        .expect("Unable to generate bindings");

    let rc = rc.borrow();
    let relevant: Vec<_> =
        rc
            .iter()
            .filter(|(key, _)| key.starts_with("jit_"))
            .map(|(key, value)| {
                (key.as_str(), std::str::from_utf8(value).unwrap())
            })
            .filter(|(_, value)| value.contains("jit_new_node"))
            .collect();

    let output = make_printable(parse_macros(&relevant));
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

    Ok(())
}
