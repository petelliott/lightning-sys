use lightning_sys::{Jit, JitWord, JitPointer, Reg};
use std::ffi::CString;
use std::convert::TryInto;

// Create constants with the same names as in the original example, to ease
// comparison between the Rust and C versions.
const JIT_R1: Reg = Reg::R(1);

#[allow(clippy::print_literal)]
#[allow(non_snake_case)]
fn main() {
    let mut j = Jit::new();
    let mut js = j.new_state();

    // make sure this outlives any calls
    let cs = CString::new("generated %d bytes\n").unwrap();

    let start = js.note(Some(file!()), line!());
    js.prolog();
    let inp = js.arg();
    js.getarg(JIT_R1, &inp);
    js.prepare();
    js.pushargi(cs.as_ptr() as JitWord);
    js.ellipsis();
    js.pushargr(JIT_R1);
    js.finishi(libc::printf as JitPointer);
    js.ret();
    js.epilog();
    let end = js.note(Some(file!()), line!());

    let myFunction = unsafe{ js.emit::<extern fn(JitWord)>() };

    /* call the generated code, passing its size as argument */
    myFunction((js.address(&end) as usize - js.address(&start) as usize).try_into().unwrap());
    js.clear_state();

    // js.disassemble(); // TODO support disassembly
}

