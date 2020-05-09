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
}
