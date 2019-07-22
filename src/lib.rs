#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod bindings;

use std::os::raw;
use std::ptr;
use std::sync::atomic;
use std::option::Option;

#[derive(Debug)]
pub struct Jit;

static JIT_MADE: atomic::AtomicBool = atomic::AtomicBool::new(false);

impl Jit {
    pub fn new() -> Option<Jit> {
        if JIT_MADE.swap(true, atomic::Ordering::Relaxed) {
            return None;
        }
        unsafe {
            //TODO: figure out how to get ptr to argv[0]
            bindings::init_jit(ptr::null::<raw::c_char>());
        }
        Some(Jit{})
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

    #[test]
    fn test_jit() {
        {
            let _jit = Jit::new().unwrap();
            match Jit::new() {
                Some(_) => assert!(false),
                None => (),
            }
        }

        {
            let _jit = Jit::new().unwrap();
            match Jit::new() {
                Some(_) => assert!(false),
                None => (),
            }
        }
    }
}
