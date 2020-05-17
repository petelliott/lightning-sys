use libc::c_int;

use lightning_sys::*;

use core::mem::size_of;

fn stack_push(js: &mut JitState, reg: Reg, sp: &mut c_int) {
    js.stxi_i((*sp).into(), Reg::FP, reg);
    *sp += size_of::<c_int>() as c_int;
}

fn stack_pop(js: &mut JitState, reg: Reg, sp: &mut c_int) {
    *sp -= size_of::<c_int>() as c_int;
    js.ldxi_i(reg, Reg::FP, (*sp).into());
}

fn compile_rpn<'a>(js: &mut JitState<'a>, mut expr: &str) -> JitNode<'a> {
    let func = js.note(None, 0);
    js.prolog();
    let inp = js.arg();
    let stack_base = js.allocai(32 * size_of::<c_int>() as c_int);
    let mut stack_ptr = stack_base;

    js.getarg_i(Reg::R(2), &inp);

    while ! expr.is_empty() {
        if expr.starts_with(|c: char| c.is_ascii_digit()) {
            let s: String = expr.chars().take_while(char::is_ascii_digit).collect();
            let val = s.parse::<JitWord>().unwrap();
            expr = &expr[s.len()..];
            stack_push(js, Reg::R(0), &mut stack_ptr);
            js.movi(Reg::R(0), val);
        } else {
            match expr.as_bytes().get(0) {
                Some(b'x') => {
                    stack_push(js, Reg::R(0), &mut stack_ptr);
                    js.movr(Reg::R(0), Reg::R(2));
                },
                Some(b'+') => {
                    stack_pop(js, Reg::R(1), &mut stack_ptr);
                    js.addr(Reg::R(0), Reg::R(1), Reg::R(0));
                },
                Some(b'-') => {
                    stack_pop(js, Reg::R(1), &mut stack_ptr);
                    js.subr(Reg::R(0), Reg::R(1), Reg::R(0));
                },
                Some(b'*') => {
                    stack_pop(js, Reg::R(1), &mut stack_ptr);
                    js.mulr(Reg::R(0), Reg::R(1), Reg::R(0));
                },
                Some(b'/') => {
                    stack_pop(js, Reg::R(1), &mut stack_ptr);
                    js.divr(Reg::R(0), Reg::R(1), Reg::R(0));
                },
                _ => panic!("cannot compile: {}", expr),
            }
            expr = &expr[1..];
        }
    }
    js.retr(Reg::R(0));
    js.epilog();

    return func;
}

fn main() {
    let j = Jit::new();
    let mut js = j.new_state();

    let nc = compile_rpn(&mut js, "32x9*5/+");
    let nf = compile_rpn(&mut js, "x32-5*9/");

    let _ = js.raw_emit();

    unsafe fn to_func<T,R>(ptr: JitPointer) -> extern "C" fn(T) -> R {
        *(&ptr as *const *mut core::ffi::c_void as *const extern "C" fn(T) -> R)
    }

    let c2f = js.address(&nc);
    let c2f = unsafe { to_func::<_, c_int>(c2f) };
    let f2c = js.address(&nf);
    let f2c = unsafe { to_func::<_, c_int>(f2c) };
    js.clear();

    print!("\nC:");
    for i in 0..=10 { print!("{:3} ", i * 10); }
    print!("\nF:");
    for i in 0..=10 { print!("{:3} ", c2f(i * 10)); }
    print!("\n");

    print!("\nF:");
    for i in 0..=10 { print!("{:3} ", i * 18 + 32); }
    print!("\nC:");
    for i in 0..=10 { print!("{:3} ", f2c(i * 18 + 32)); }
    print!("\n");
}
