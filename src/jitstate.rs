use crate::bindings;

#[derive(Debug, PartialEq, Eq)]
pub struct JitState<'a> {
    pub(crate) state: *mut bindings::jit_state_t,
    pub(crate) phantom: std::marker::PhantomData<&'a ()>,
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
}
