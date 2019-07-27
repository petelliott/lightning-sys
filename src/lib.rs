#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod bindings;

use std::os::raw;
use std::ptr;
use std::sync::atomic;

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
            phantom: std::marker::PhantomData,
        }
    }

    pub fn r_num() -> i32 {
        unsafe {
            bindings::lgsys_JIT_R_NUM
        }
    }

    pub fn v_num() -> i32 {
        unsafe {
            bindings::lgsys_JIT_V_NUM
        }
    }

    pub fn f_num() -> i32 {
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

pub enum Reg {
    R(i32),
    V(i32),
    F(i32),
}

#[derive(Debug, PartialEq, Eq)]
pub struct JitState<'a> {
    state: *mut bindings::jit_state_t,
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Drop for JitState<'a> {
    fn drop(&mut self) {
        unsafe {
            bindings::_jit_destroy_state(self.state);
        }
    }
}

impl<'a> JitState<'a> {
    pub fn clear(&mut self) {
        unsafe {
            bindings::_jit_clear_state(self.state);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Jit;

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
    fn test_jitstate() {
        let jit = Jit::new();
        let mut js1 = jit.new_state();
        js1.clear();
        let _js2 = jit.new_state();
    }
}
