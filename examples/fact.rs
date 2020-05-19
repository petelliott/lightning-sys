use lightning_sys::{Jit, JitWord, Reg, NULL};

// Create constants with the same names as in the original example, to ease
// comparison between the Rust and C versions.
const JIT_R0: Reg = Reg::R(0);
const JIT_R1: Reg = Reg::R(1);

#[allow(clippy::print_literal)]
fn main() {
    let mut j = Jit::new();
    let mut js = j.new_state();

    /* declare a forward label */
    let fact = js.forward();

    js.prolog();                        /* Entry point of the factorial function */
    let inp = js.arg();                 /* Receive an integer argument */
    js.getarg(JIT_R0, &inp);            /* Move argument to RO */
    js.prepare();
    js.pushargi(1);                     /* This is the accumulator */
    js.pushargr(JIT_R0);                /* This is the argument */
    let call = js.finishi(NULL);        /* Call the tail call optimized function */
    js.patch_at(&call, &fact);          /* Patch call to forward defined function */
    /* the above could have been written as:
     *          js.patch_at(js.finishi(NULL), fact);
     */
    js.retval(JIT_R0);                  /* Fetch the result */
    js.retr(JIT_R0);                    /* Return it */
    js.epilog();                        /* Epilog *before* label before prolog */

    /* define the forward label */
    js.link(&fact);                     /* Entry point of the helper function */
    js.prolog();
    js.frame(16);                       /* Reserve 16 bytes in the stack */
    let fact_entry = js.label();        /* This is the tail call entry point */
    let ac = js.arg();                  /* The accumulator is the first argument */
    let inp = js.arg();                 /* The factorial argument */
    js.getarg(JIT_R0, &ac);             /* Move the accumulator to R0 */
    js.getarg(JIT_R1, &inp);            /* Move the argument to R1 */
    let fact_out = js.blei(JIT_R1, 1);  /* Done if argument is one or less */
    js.mulr(JIT_R0, JIT_R0, JIT_R1);    /* accumulator *= argument */
    js.putargr(JIT_R0, &ac);            /* Update the accumulator */
    js.subi(JIT_R1, JIT_R1, 1);         /* argument -= 1 */
    js.putargr(JIT_R1, &inp);           /* Update the argument */
    let jump = js.jmpi();
    js.patch_at(&jump, &fact_entry);    /* Tail Call Optimize it! */
    js.patch(&fact_out);
    js.retr(JIT_R0);                    /* Return the accumulator */

    let factorial = unsafe{ js.emit::<extern fn(JitWord) -> JitWord>() };
    /* no need to query information about resolved addresses */
    js.clear();

    let arg = std::env::args().nth(1).map(|x| x.parse().unwrap_or(0)).unwrap_or(5);

    /* call the generated code */
    println!("factorial({}) = {}", arg, factorial(arg));
    /* release all memory associated with the _jit identifier */
    /* (this happens automatically with Drop in the Rust version) */
}

