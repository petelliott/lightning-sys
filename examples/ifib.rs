use lightning_sys::{Jit, JitWord, Reg};

// Create constants with the same names as in the original example, to ease
// comparison between the Rust and C versions.
const JIT_R0: Reg = Reg::R(0);
const JIT_R1: Reg = Reg::R(1);
const JIT_R2: Reg = Reg::R(2);
const JIT_V0: Reg = Reg::V(0);

#[allow(clippy::print_literal)]
fn main() {
    let mut j = Jit::new();
    let mut js = j.new_state();

                js.prolog   ();
    let inp =   js.arg      ();
                js.getarg   (JIT_R0, &inp);             /* R0 = n */
    let zero =  js.beqi     (JIT_R0, 0);
                js.movr     (JIT_R1, JIT_R0);
                js.movi     (JIT_R0, 1);
    let refr =  js.blei     (JIT_R1, 2);
                js.subi     (JIT_R2, JIT_R1, 2);
                js.movr     (JIT_R1, JIT_R0);

    let top =   js.label();
                js.subi     (JIT_R2, JIT_R2, 1);        /* decr. counter */
                js.movr     (JIT_V0, JIT_R0);           /* V0 = R0 */
                js.addr     (JIT_R0, JIT_R0, JIT_R1);   /* R0 = R0 + R1 */
                js.movr     (JIT_R1, JIT_V0);           /* R1 = V0 */
    let jump =  js.bnei     (JIT_R2, 0);                /* if (R2) goto loop; */
    js.patch_at(&jump, &top);

    js.patch(&refr);                                    /* patch forward jump */
    js.patch(&zero);                                    /* patch forward jump */
                js.retr     (JIT_R0);

    let fib = unsafe{ js.emit::<extern fn(JitWord) -> JitWord>() };
    js.clear();

    println!("fib({}) = {}", 36, fib(36));
}

