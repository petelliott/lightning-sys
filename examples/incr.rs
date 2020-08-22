use lightning_sys::{Jit, JitWord, Reg};

// Create constants with the same names as in the original example, to ease
// comparison between the Rust and C versions.
const JIT_R0: Reg = Reg::R(0);

#[allow(clippy::print_literal)]
fn main() {
    let mut j = Jit::new();
    let mut js = j.new_state();

    js.prolog();
    let inp = js.arg();
    js.getarg(JIT_R0, &inp);
    js.addi(JIT_R0, JIT_R0, 1);
    js.retr(JIT_R0);

    let incr = unsafe{ js.cast_emit::<extern fn(JitWord) -> JitWord>() };
    js.clear_state();

    /* call the generated code, passing 5 as an argument */
    println!("{} + 1 = {}", 5, incr(5));
}

