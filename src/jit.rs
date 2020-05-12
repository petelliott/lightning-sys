#![allow(clippy::mutex_atomic)] // Avoid clippy warning about JITS_MADE
#![allow(clippy::new_without_default)] // Avoid clippy warning about Jit::new

use std::os::raw;
use std::ptr;
use std::sync::Mutex;

use crate::bindings;
use crate::JitState;


#[derive(Debug)]
pub struct Jit;

lazy_static! {
    static ref JITS_MADE: Mutex<usize> = Mutex::new(0);
}

impl Jit {
    pub fn new() -> Jit {
        let mut m = JITS_MADE.lock().unwrap();

        if *m == 0 {
            unsafe {
                //TODO: figure out how to get ptr to argv[0]
                bindings::init_jit(ptr::null::<raw::c_char>());
            }
        }

        *m += 1;
        Jit{}
    }

    pub fn new_state(&self) -> JitState {
        JitState {
            state: unsafe {
                bindings::jit_new_state()
            },
            jit: &self,
        }
    }

    pub fn r_num(&self) -> bindings::jit_gpr_t {
        unsafe {
            bindings::lgsys_JIT_R_NUM
        }
    }

    pub fn v_num(&self) -> bindings::jit_gpr_t {
        unsafe {
            bindings::lgsys_JIT_V_NUM
        }
    }

    pub fn f_num(&self) -> bindings::jit_gpr_t {
        unsafe {
            bindings::lgsys_JIT_F_NUM
        }
    }

}

impl Drop for Jit {
    fn drop(&mut self) {
        let mut m = JITS_MADE.lock().unwrap();
        *m -= 1;

        if *m == 0 {
            unsafe {
                bindings::finish_jit();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Jit;
    use crate::Reg;
    use crate::types::ToFFI;

    #[test]
    fn test_jit() {
        {
            let _jit = Jit::new();
            Jit::new();
        }

        {
            let _jit = Jit::new();
            Jit::new();
        }

    }

    #[test]
    fn test_reg_num() {
        let jit = Jit::new();
        assert!(jit.r_num() >= 3);
        assert!(jit.v_num() >= 3);
        assert!(jit.f_num() >= 6);
    }

    #[test]
    fn test_to_ffi() {
        let jit = Jit::new();

        assert!(std::panic::catch_unwind(|| Reg::R(jit.r_num()).to_ffi()).is_err());
        Reg::R(jit.r_num()-1).to_ffi();
        Reg::R(0).to_ffi();

        assert!(std::panic::catch_unwind(|| Reg::V(jit.v_num()).to_ffi()).is_err());
        Reg::V(jit.v_num()-1).to_ffi();
        Reg::V(0).to_ffi();

        assert!(std::panic::catch_unwind(|| Reg::F(jit.f_num()).to_ffi()).is_err());
        Reg::F(jit.f_num()-1).to_ffi();
        Reg::F(0).to_ffi();
    }

    #[test]
    fn test_printf() {
        use std::ffi::CString;
        use crate::{Jit, JitWord, Reg, JitPointer};
        use std::convert::TryInto;

        let jit = Jit::new();
        let js = jit.new_state();

        // make sure this outlives any calls
        let cs = CString::new("generated %d bytes\n").unwrap();

        let start = js.note(file!(), line!());
        js.prolog();
        let inarg = js.arg();
        js.getarg(Reg::R(1), &inarg);
        js.prepare();
        js.pushargi(cs.as_ptr() as JitWord);
        js.ellipsis();
        js.pushargr(Reg::R(1));
        js.finishi(libc::printf as JitPointer);
        js.ret();
        js.epilog();
        let end = js.note(file!(), line!());

        let my_function = unsafe{ js.emit::<extern fn(JitWord)>() };
        /* call the generated code, passing its size as argument */
        my_function((js.address(&end) as u64 - js.address(&start) as u64).try_into().unwrap());
        js.clear();

        // TODO: dissasembly has not been implemented yet
        // js.dissasemble();
    }

    #[test]
    #[allow(clippy::print_literal)]
    fn test_fibonacci() {
        use crate::{Jit, JitWord, Reg, NULL};

        let jit = Jit::new();
        let js = jit.new_state();

        let label = js.label();
                    js.prolog();
        let inarg = js.arg();
                    js.getarg(Reg::R(0), &inarg);
        let zero  = js.beqi(Reg::R(0), 0);
                    js.movr(Reg::V(0), Reg::R(0));
                    js.movi(Reg::R(0), 1);
        let refr  = js.blei(Reg::V(0), 2);
                    js.subi(Reg::V(1), Reg::V(0), 1);
                    js.subi(Reg::V(2), Reg::V(0), 2);
                    js.prepare();
                    js.pushargr(Reg::V(1));
        let call  = js.finishi(NULL);
                    js.patch_at(&call, &label);
                    js.retval(Reg::V(1));
                    js.prepare();
                    js.pushargr(Reg::V(2));
        let call2 = js.finishi(NULL);
                    js.patch_at(&call2, &label);
                    js.retval(Reg::R(0));
                    js.addr(Reg::R(0), Reg::R(0), Reg::V(1));

                    js.patch(&refr);
                    js.patch(&zero);
                    js.retr(Reg::R(0));
                    js.epilog();

        let fib = unsafe{ js.emit::<extern fn(JitWord) -> JitWord>() };
        js.clear();

        println!("fib({})={}", 32, fib(32));
        assert_eq!(0, fib(0));
        assert_eq!(1, fib(1));
        assert_eq!(1, fib(2));
        assert_eq!(2_178_309, fib(32));
    }

    #[test]
    #[allow(clippy::print_literal)]
    fn test_factorial() {
        use crate::{Jit, JitWord, Reg, NULL};

        let jit = Jit::new();
        let js = jit.new_state();

        let fact = js.forward();

                    js.prolog();
        let inarg = js.arg();
                    js.getarg(Reg::R(0), &inarg);
                    js.prepare();
                    js.pushargi(1);
                    js.pushargr(Reg::R(0));
        let call  = js.finishi(NULL);
                    js.patch_at(&call, &fact);

                    js.retval(Reg::R(0));
                    js.retr(Reg::R(0));
                    js.epilog();

        js.link(&fact);
                    js.prolog();
                    js.frame(16);
        let f_ent = js.label(); // TCO entry point
        let ac    = js.arg();
        let ina   = js.arg();
                    js.getarg(Reg::R(0), &ac);
                    js.getarg(Reg::R(1), &ina);
        let f_out = js.blei(Reg::R(1), 1);
                    js.mulr(Reg::R(0), Reg::R(0), Reg::R(1));
                    js.putargr(Reg::R(0), &ac);
                    js.subi(Reg::R(1), Reg::R(1), 1);
                    js.putargr(Reg::R(1), &ina);
        let jump  = js.jmpi(); // tail call optimiation
                    js.patch_at(&jump, &f_ent);
                    js.patch(&f_out);
                    js.retr(Reg::R(0));

        let factorial = unsafe{ js.emit::<extern fn(JitWord) -> JitWord>() };
        js.clear();

        println!("factorial({}) = {}", 5, factorial(5));
        assert_eq!(1, factorial(1));
        assert_eq!(2, factorial(2));
        assert_eq!(6, factorial(3));
        assert_eq!(24, factorial(4));
        assert_eq!(120, factorial(5));
    }
}
