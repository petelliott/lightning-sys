use crate::bindings;
use crate::Jit;
use crate::JitNode;
use crate::{JitWord, JitUword, JitPointer};

#[derive(Debug)]
pub struct JitState<'a> {
    pub(crate) state: *mut bindings::jit_state_t,
    pub(crate) jit: &'a Jit,
}

impl<'a> Drop for JitState<'a> {
    fn drop(&mut self) {
        unsafe {
            bindings::_jit_destroy_state(self.state);
        }
    }
}

// implementations of utility functions
impl<'a> JitState<'a> {
    pub fn clear(&mut self) {
        unsafe {
            bindings::_jit_clear_state(self.state);
        }
    }
}

macro_rules! jit_impl {
    ( $op:ident, _ ) => { jit_impl_inner!($op, e); };

    ( $op:ident, w ) => { jit_impl_inner!($op, e_w, a: JitWord); };
    ( $op:ident, f ) => { jit_impl_inner!($op, e_f, a: f32); };
    ( $op:ident, d ) => { jit_impl_inner!($op, e_d, a: f64); };
    ( $op:ident, p ) => { jit_impl_inner!($op, e_p, a: JitPointer); };

    ( $op:ident, ww ) => { jit_impl_inner!($op, e_ww, a: JitWord, b: JitWord); };
    ( $op:ident, wp ) => { jit_impl_inner!($op, e_wp, a: JitWord, b: JitPointer); };
    ( $op:ident, fp ) => { jit_impl_inner!($op, e_fp, a: f32, b: JitPointer); };
    ( $op:ident, dp ) => { jit_impl_inner!($op, e_dp, a: f64, b: JitPointer); };
    ( $op:ident, pw ) => { jit_impl_inner!($op, e_pw, a: JitPointer, b: JitWord); };
    ( $op:ident, wf ) => { jit_impl_inner!($op, e_wf, a: JitWord, b: f32); };
    ( $op:ident, wd ) => { jit_impl_inner!($op, e_wd, a: JitWord, b: f64); };

    ( $op:ident, www ) => { jit_impl_inner!($op, e_www, a: JitWord, b: JitWord, c: JitWord); };
    ( $op:ident, wwf ) => { jit_impl_inner!($op, e_wwf, a: JitWord, b: JitWord, c: f32); };
    ( $op:ident, wwd ) => { jit_impl_inner!($op, e_wwd, a: JitWord, b: JitWord, c: f64); };
    ( $op:ident, pww ) => { jit_impl_inner!($op, e_pww, a: JitPointer, b: JitWord, c: JitWord); };
    ( $op:ident, pwf ) => { jit_impl_inner!($op, e_pwf, a: JitPointer, b: JitWord, c: f32); };
    ( $op:ident, pwd ) => { jit_impl_inner!($op, e_pwd, a: JitPointer, b: JitWord, c: f64); };

    ( $op:ident, qww ) => { jit_impl_inner!($op, e_qww, a: i32, b: i32, c: JitWord, d: JitWord); };
}

macro_rules! jit_function {
    ( $form:ident ) => {{
        mashup! {
            m["method"] = _jit_new_nod $form;
        }
        m! {
            bindings::"method"
        }
    }};
}

macro_rules! jit_code {
    ( $form:ident ) => {{
        mashup! {
            m["code"] = jit_code_t_jit_code_ $form;
        }
        m! {
            bindings::"code"
        }
    }};
}

macro_rules! jit_impl_inner {
    ( $op:ident, $ifmt:ident $(, $arg:ident: $type:ty)* ) => {
        pub fn $op<'b>(&'b mut self $(, $arg: $type)*) -> JitNode<'b> {
            JitNode{
                node: unsafe { jit_function!($ifmt)(self.state, jit_code!($op) $(, $arg)*) },
                state: &*self,
            }
        }
    };
}

// implmentations of instructions
impl<'a> JitState<'a> {
    jit_impl!(live, w);
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
