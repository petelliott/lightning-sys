use std::os::raw;
use std::ptr;
use std::sync::atomic;

use crate::bindings;
use crate::JitState;
use crate::Reg;

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

    pub fn r_num(&self) -> i32 {
        unsafe {
            bindings::lgsys_JIT_R_NUM
        }
    }

    pub fn v_num(&self) -> i32 {
        unsafe {
            bindings::lgsys_JIT_V_NUM
        }
    }

    pub fn f_num(&self) -> i32 {
        unsafe {
            bindings::lgsys_JIT_F_NUM
        }
    }

    pub(crate) fn _get_reg(&self, r: Reg) -> i32 {
        match r {
            Reg::R(i) => if i < self.r_num() {
                unsafe { bindings::lgsys_jit_r(i) }
            } else {
                panic!("register 'R{}' is not supported", i);
            },

            Reg::V(i) => if i < self.v_num() {
                unsafe { bindings::lgsys_jit_v(i) }
            } else {
                panic!("register 'R{}' is not supported", i);
            },

            Reg::F(i) => if i < self.f_num() {
                unsafe { bindings::lgsys_jit_f(i) }
            } else {
                panic!("register 'R{}' is not supported", i);
            },
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
    fn test_get_reg() {
        let jit = Jit::new();

        assert!(std::panic::catch_unwind(|| jit._get_reg(Reg::R(jit.r_num()))).is_err());
        jit._get_reg(Reg::R(jit.r_num()-1));
        jit._get_reg(Reg::R(0));

        assert!(std::panic::catch_unwind(|| jit._get_reg(Reg::V(jit.v_num()))).is_err());
        jit._get_reg(Reg::V(jit.v_num()-1));
        jit._get_reg(Reg::V(0));

        assert!(std::panic::catch_unwind(|| jit._get_reg(Reg::F(jit.f_num()))).is_err());
        jit._get_reg(Reg::F(jit.f_num()-1));
        jit._get_reg(Reg::F(0));
    }
}
