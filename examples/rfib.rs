use lightning_sys::{Jit, JitWord, Reg, NULL};

// Create constants with the same names as in the original example, to ease
// comparison between the Rust and C versions.
const JIT_R0: Reg = Reg::R(0);
const JIT_V0: Reg = Reg::V(0);
const JIT_V1: Reg = Reg::V(1);
const JIT_V2: Reg = Reg::V(2);

#[allow(clippy::print_literal)]
fn main() {
    let mut j = Jit::new();
    let mut js = j.new_state();

    let label = js.label    ();
                js.prolog   ();
    let inp =   js.arg      ();
                js.getarg   (JIT_R0, &inp);             /* R0 = n */
    let zero =  js.beqi     (JIT_R0, 0);
                js.movr     (JIT_V0, JIT_R0);           /* V0 = R0 */
                js.movi     (JIT_R0, 1);
    let refr =  js.blei     (JIT_V0, 2);
                js.subi     (JIT_V1, JIT_V0, 1);        /* V1 = n-1 */
                js.subi     (JIT_V2, JIT_V0, 2);        /* V2 = n-2 */
                js.prepare();
                js.pushargr(JIT_V1);
    let call =  js.finishi(NULL);
                js.patch_at(&call, &label);
                js.retval(JIT_V1);                      /* V1 = fib(n-1) */
                js.prepare();
                js.pushargr(JIT_V2);
    let call =  js.finishi(NULL);
                js.patch_at(&call, &label);
                js.retval(JIT_R0);                      /* R0 = fib(n-2) */
                js.addr(JIT_R0, JIT_R0, JIT_V1);        /* R0 = R0 + V1 */

    js.patch(&refr);                                    /* patch jump */
    js.patch(&zero);                                    /* patch jump */
                js.retr(JIT_R0);

    let fib = unsafe{ js.cast_emit::<extern fn(JitWord) -> JitWord>() };
    js.clear_state();

    println!("fib({}) = {}", 32, fib(32));
}

