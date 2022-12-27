extern crate bindgen;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::env;
use std::io::Write;
use std::ops::Index;
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;
use std::rc::Rc;

// We need the interior mutability of `RefCell` to work around the fact that the
// `func_macro` callback is not called with a `mut` reference. We need
// `AssertUnwindSafe` to allow `RefCell` to be used within a `catch_unwind`
// context inside `bindgen`. We use `Rc` to so that we can get back the `T`
// after `bindgen` is done with the `Callbacks`, by keeping the reference alive
// without having to make it `&'static` instead.
type HideUnwinding<T> = Rc<AssertUnwindSafe<RefCell<T>>>;

struct Callbacks {
    /// CargoCallbacks tells bindgen to regenerate bindings if header files'
    /// contents or transitively included files change.
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
        return orig.split('_').collect();
    }

    if num_underscores == 0 {
        return vec![orig];
    }

    const SUFFIXES: &[&str] = &[
        "_f", "_d",
        "_u",
        "_c", "_i", "_l", "_s",
        "_uc", "_ui", "_ul", "_us",
        "_p",
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
    inverse_variants: &BTreeMap<String,&'a str>,
    entry: &'a str,
    orig: &'a str,
) -> Option<Record> {
    let brief = &entry[..entry.find('(').unwrap()];
    let core = brief.trim_start_matches("jit_");
    inverse_variants.get(core).and_then(|iv| {
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
        Some(Record { entry, stem, pieces, orig })
    })
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
        let last_matches = (last_char == 'r' || last_char == 'i')
            && !haystack.contains('_')
            && haystack.len() > 1;
        haystack.starts_with(needle)
            && (haystack.len() - needle.len() < 2)
            && (haystack.len() == needle.len() || last_matches)
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
        .filter_map(|(k, v)| extract(&variants, &inverse_variants, k, &v))
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
    use std::io::{BufRead, BufReader};
    use std::fs::File;

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let base = PathBuf::from("vendor/gnu-lightning");
    let incdir = base.join("include");
    let libdir = base.join("lib");

    {
        let input = BufReader::new(File::open(incdir.join("lightning.h.in"))?);
        let mut output = File::create(out_path.join("lightning.h"))?;
        for line in input.lines() {
            let line = line?;
            match line.as_str() {
                "@MAYBE_INCLUDE_STDINT_H@" => writeln!(output, "#include <stdint.h>")?,
                _ if !line.contains('@') => writeln!(output, "{}", line)?,
                _ => panic!("Found unexpected automake token in lightning header"),
            }
        }
    }

    let definitions = &[
        // Without HAVE_MMAP, we crash.
        // TODO make HAVE_MMAP dependent on target
        ("HAVE_MMAP", None),
        // TODO make HAVE_FFSL dependent on target (it is in POSIX, but Windows
        // might not have it)
        ("HAVE_FFSL", None),
        // TODO remove NDEBUG -- it works works around an invalid assertion
        // (`assert(l != h)`) in _jit_new_node_qww
        ("NDEBUG", None),
    ];

    let files = &[
        libdir.join("jit_disasm.c"),
        libdir.join("jit_memory.c"),
        libdir.join("jit_names.c"),
        libdir.join("jit_note.c"),
        libdir.join("jit_print.c"),
        libdir.join("jit_size.c"),
        libdir.join("lightning.c"),
    ];

    let mut builder = cc::Build::new();

    for &(name, val) in definitions {
        builder.define(name, val);
    }

    for file in files {
        builder.file(file);
        // N.B.: This does not catch .c and .h files that are #include'd by the
        // top-level files, but doing this is better than nothing.
        println!("cargo:rerun-if-changed={}", file.to_str().unwrap());
    }

    println!("cargo:rerun-if-changed={}", "C/register.c");
    println!("cargo:rerun-if-changed={}", "C/lightning-sys.h");

    builder
        .include(incdir.clone())
        .include(out_path.clone())
        .file("C/register.c")
        .flag_if_supported("-Wno-unused")
        .flag_if_supported("-Wno-unused-parameter")
        .compile("lightningsys");

    let bt = BTreeMap::new();
    let ce = AssertUnwindSafe(RefCell::new(bt));
    let rc = Rc::new(ce);
    let cb = Callbacks::new(Rc::clone(&rc));

    let bindings = bindgen::Builder::default()
        .header(out_path.join("lightning.h").to_str().unwrap())
        .header("C/lightning-sys.h")
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
