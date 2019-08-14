use std::os::raw;
use std::ptr;
use std::sync::atomic;

use crate::bindings;
use crate::JitState;

#[derive(Debug)]
pub struct Jit;

static JIT_MADE: atomic::AtomicBool = atomic::AtomicBool::new(false);

impl Jit {
    pub fn new() -> Jit {
        if JIT_MADE.swap(true, atomic::Ordering::Relaxed) {
            panic!("there is already an instance of Jit created");
        }
        unsafe {
            //TODO: figure out how to get ptr to argv[0]
            bindings::init_jit(ptr::null::<raw::c_char>());
        }
        Jit{}
    }

    pub fn new_state<'a>(&'a self) -> JitState<'a> {
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
        unsafe {
            bindings::finish_jit();
        }
        JIT_MADE.store(false, atomic::Ordering::Relaxed);
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
            assert!(std::panic::catch_unwind(|| Jit::new()).is_err());
        }

        {
            let _jit = Jit::new();
            assert!(std::panic::catch_unwind(|| Jit::new()).is_err());
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
}
