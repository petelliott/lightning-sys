#![allow(clippy::mutex_atomic)] // Avoid clippy warning about JITS_MADE
#![allow(clippy::new_without_default)] // Avoid clippy warning about Jit::new
#![deny(unused_must_use)]

use std::sync::Mutex;

use crate::bindings;
use crate::JitState;

use std::marker::PhantomData;

#[derive(Debug)]
pub struct Jit<'a>(PhantomData<&'a ()>);

lazy_static! {
    static ref JITS_MADE: Mutex<usize> = Mutex::new(0);
}

impl<'a> Jit<'a> {
    #[must_use]
    pub fn new() -> Jit<'a> {
        let mut m = JITS_MADE.lock().unwrap();

        if *m == 0 {
            unsafe {
                //TODO: figure out how to get ptr to argv[0]
                bindings::init_jit(std::ptr::null());
            }
        }

        *m += 1;
        Jit(PhantomData)
    }

    // This takes &mut self instead of &self because the unsafe operations wrapped herein are
    // inherently mutating.
    #[must_use]
    pub fn new_state(&mut self) -> JitState {
        JitState {
            state: unsafe {
                bindings::jit_new_state()
            },
            phantom: PhantomData,
        }
    }

    #[must_use]
    pub fn r_num(&self) -> bindings::jit_gpr_t {
        unsafe {
            bindings::lgsys_JIT_R_NUM()
        }
    }

    #[must_use]
    pub fn v_num(&self) -> bindings::jit_gpr_t {
        unsafe {
            bindings::lgsys_JIT_V_NUM()
        }
    }

    #[must_use]
    pub fn f_num(&self) -> bindings::jit_gpr_t {
        unsafe {
            bindings::lgsys_JIT_F_NUM()
        }
    }

}

impl<'a> Drop for Jit<'a> {
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
            let _ = Jit::new();
        }

        {
            let _jit = Jit::new();
            let _ = Jit::new();
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
        let _ = Reg::R(jit.r_num()-1).to_ffi();
        let _ = Reg::R(0).to_ffi();

        assert!(std::panic::catch_unwind(|| Reg::V(jit.v_num()).to_ffi()).is_err());
        let _ = Reg::V(jit.v_num()-1).to_ffi();
        let _ = Reg::V(0).to_ffi();

        assert!(std::panic::catch_unwind(|| Reg::F(jit.f_num()).to_ffi()).is_err());
        let _ = Reg::F(jit.f_num()-1).to_ffi();
        let _ = Reg::F(0).to_ffi();
    }
}
