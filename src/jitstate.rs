use crate::bindings;
use crate::Jit;
use crate::Reg;
use crate::JitNode;
use crate::{JitWord, JitPointer};
use crate::ToFFI;
use std::ffi::{CString, c_void};
use std::ptr::null_mut;

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
    pub fn clear(&self) {
        unsafe {
            bindings::_jit_clear_state(self.state);
        }
    }

    // there is no way to require a function type in a trait bound
    // without specifying the number of arguments
    pub unsafe fn emit<T: Copy>(&self) -> T {
        *std::mem::transmute::<&JitPointer, &T>(
            &bindings::_jit_emit(self.state)
        )
    }

    pub fn raw_emit(&self) -> JitPointer {
        unsafe {
            bindings::_jit_emit(self.state)
        }
    }
}

macro_rules! jit_impl {
    ( $op:ident, _ ) => { jit_impl_inner!($op, _); };

    ( $op:ident, w ) => { jit_impl_inner!($op, w, a: Reg => JitWord); };
    //( $op:ident, f ) => { jit_impl_inner!($op, f, a: Reg => JitWord); };
    //( $op:ident, d ) => { jit_impl_inner!($op, d, a: Reg => JitWord); };
    //( $op:ident, p ) => { jit_impl_inner!($op, p, a: Reg => JitWord); };

    ( $op:ident, i_w ) => { jit_impl_inner!($op, w, a: JitWord => _); };
    ( $op:ident, i_f ) => { jit_impl_inner!($op, f, a: f32 => _); };
    ( $op:ident, i_d ) => { jit_impl_inner!($op, d, a: f64 => _); };
    ( $op:ident, i_p ) => { jit_impl_inner!($op, p, a: JitPointer => _); };

    ( $op:ident, ww ) => { jit_impl_inner!($op, ww, a: Reg => JitWord, b: Reg => JitWord); };
    //( $op:ident, wp ) => { jit_impl_inner!($op, wp, a: Reg => JitWord, b: Reg => JitWord); };
    //( $op:ident, fp ) => { jit_impl_inner!($op, fp, a: Reg => JitWord, b: Reg => JitWord); };
    //( $op:ident, dp ) => { jit_impl_inner!($op, dp, a: Reg => JitWord, b: Reg => JitWord); };
    //( $op:ident, pw ) => { jit_impl_inner!($op, pw, a: Reg => JitWord, b: Reg => JitWord); };
    //( $op:ident, wf ) => { jit_impl_inner!($op, wf, a: Reg => JitWord, b: Reg => JitWord); };
    //( $op:ident, wd ) => { jit_impl_inner!($op, wd, a: Reg => JitWord, b: Reg => JitWord); };

    ( $op:ident, i_ww ) => { jit_impl_inner!($op, ww, a: Reg => JitWord, b: JitWord => _); };
    ( $op:ident, i_wp ) => { jit_impl_inner!($op, wp, a: Reg => JitWord, b: JitPointer => _); };
    ( $op:ident, i_fp ) => { jit_impl_inner!($op, fp, a: Reg => JitWord, b: JitPointer => _); };
    ( $op:ident, i_dp ) => { jit_impl_inner!($op, dp, a: Reg => JitWord, b: JitPointer => _); };
    ( $op:ident, i_pw ) => { jit_impl_inner!($op, pw, a: Reg => JitWord, b: JitWord => _); };
    ( $op:ident, i_wf ) => { jit_impl_inner!($op, wf, a: Reg => JitWord, b: f32 => _); };
    ( $op:ident, i_wd ) => { jit_impl_inner!($op, wd, a: Reg => JitWord, b: f64 => _); };

    ( $op:ident, www ) => { jit_impl_inner!($op, www, a: Reg => JitWord, b: Reg => JitWord, c: Reg => JitWord); };
    //( $op:ident, wwf ) => { jit_impl_inner!($op, wwf, a: Reg => JitWord, b: Reg => JitWord, c: Reg => JitWord); };
    //( $op:ident, wwd ) => { jit_impl_inner!($op, wwd, a: Reg => JitWord, b: Reg => JitWord, c: Reg => JitWord); };
    //( $op:ident, pww ) => { jit_impl_inner!($op, pww, a: Reg => JitWord, b: Reg => JitWord, c: Reg => JitWord); };
    //( $op:ident, pwf ) => { jit_impl_inner!($op, pwf, a: Reg => JitWord, b: Reg => JitWord, c: Reg => JitWord); };
    //( $op:ident, pwd ) => { jit_impl_inner!($op, pwd, a: Reg => JitWord, b: Reg => JitWord, c: Reg => JitWord); };

    ( $op:ident, i_www ) => { jit_impl_inner!($op, www, a: Reg => JitWord, b: Reg => JitWord, c: JitWord => _); };
    ( $op:ident, i_wwf ) => { jit_impl_inner!($op, wwf, a: Reg => JitWord, b: Reg => JitWord, c: f32 => _); };
    ( $op:ident, i_wwd ) => { jit_impl_inner!($op, wwd, a: Reg => JitWord, b: Reg => JitWord, c: f64 => _); };
    ( $op:ident, i_pww ) => { jit_impl_inner!($op, pww, a: Reg => JitWord, b: Reg => JitWord, c: JitWord => _); };
    ( $op:ident, i_pwf ) => { jit_impl_inner!($op, pwf, a: Reg => JitWord, b: Reg => JitWord, c: f32 => _); };
    ( $op:ident, i_pwd ) => { jit_impl_inner!($op, pwd, a: Reg => JitWord, b: Reg => JitWord, c: f64 => _); };

    ( $op:ident, qww ) => { jit_impl_inner!($op, qww, a: Reg => i32, b: Reg => i32, c: Reg => JitWord, d: Reg => JitWord); };
    ( $op:ident, i_qww ) => { jit_impl_inner!($op, qww, a: Reg => i32, b: Reg => i32, c: Reg => JitWord, d: JitWord => _); };
}

macro_rules! jit_new_node {
    ( _ ) => { bindings::_jit_new_node };
    ( $form:ident ) => {{
        mashup! {
            m["method"] = _jit_new_node_ $form;
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


macro_rules! jit_impl_type {
    ( $e:expr => _ ) => { $e };
    ( $e:expr => $t:ty ) => { $e as $t };
}

macro_rules! jit_impl_inner {
    ( $op:ident, $ifmt:ident $(, $arg:ident: $type:ty => $target:ty)* ) => {
        pub fn $op<'b>(&'b self $(, $arg: $type)*) -> JitNode<'b> {
            JitNode{
                node: unsafe { jit_new_node!($ifmt)(self.state, jit_code!($op) $(, jit_impl_type!($arg.to_ffi() => $target))*) },
                phantom: std::marker::PhantomData,
            }
        }
    };
    ( $op:ident, $ifmt:ident $(, $arg:ident: $type:ty)* ) => {
        jit_impl_inner!(kkk);
    };
}

macro_rules! jit_prefix {
    ( $form:ident ) => {{
        mashup! {
            m["method"] = _jit_ $form;
        }
        m! {
            bindings::"method"
        }
    }}
}

macro_rules! jit_reexport {
    ( $fn:ident $(, $arg:ident : $typ:ty )*; -> JitNode) => {
        pub fn $fn<'b>(&'b self $(, $arg: $typ )*) -> JitNode<'b> {
            JitNode{
                node: unsafe { jit_prefix!($fn)(self.state $(, $arg.to_ffi())*) },
                phantom: std::marker::PhantomData,
            }
        }
    };
    ( $fn:ident $(, $arg:ident : $typ:ty )*; -> $ret:ty) => {
        pub fn $fn<'b>(&'b self $(, $arg: $typ )*) -> $ret {
            unsafe { jit_prefix!($fn)(self.state $(, $arg.to_ffi())*) }
        }
    };
    ( $fn:ident $(, $arg:ident : $typ:ty )*) => { jit_reexport!($fn $(, $arg : $typ)*; -> ()); }
}

macro_rules! jit_imm {
    (i) => { JitWord };
    (r) => { Reg };
}


macro_rules! jit_branch {
    ( $fn:ident, $t:ident ) => {
        pub fn $fn<'b>(&'b self, a: Reg, b: jit_imm!($t)) -> JitNode<'b> {
            JitNode{
                node: unsafe{ jit_new_node!(pww)(self.state, jit_code!($fn), null_mut::<c_void>(), a.to_ffi() as JitWord, b.to_ffi() as JitWord) },
                phantom: std::marker::PhantomData,
            }
        }
    };
}

// implmentations of instructions
impl<'a> JitState<'a> {
    jit_impl!(live, w);
    jit_impl!(align, w);

    pub fn name<'b>(&'b self, name: &str) -> JitNode<'b> {
        // I looked at the lightning code, this will be copied
        let cs = CString::new(name).unwrap();
        JitNode{
            node: unsafe { bindings::_jit_name(self.state, cs.as_ptr()) },
            phantom: std::marker::PhantomData,
        }
    }

    pub fn note<'b>(&'b self, file: &str, line: u32) -> JitNode<'b> {
        // I looked at the lightning code, this will be copied
        let cs = CString::new(file).unwrap();
        JitNode{
            node: unsafe { bindings::_jit_note(self.state, cs.as_ptr(), line as i32) },
            phantom: std::marker::PhantomData,
        }
    }

    jit_reexport!(label; -> JitNode);
    jit_reexport!(forward; -> JitNode);
    jit_reexport!(indirect; -> JitNode);
    jit_reexport!(link, node: &JitNode);

    jit_reexport!(prolog);
    jit_reexport!(ellipsis);

    jit_reexport!(allocai, size: i32; -> i32);
    jit_reexport!(allocar, off: Reg, size: Reg);

    jit_reexport!(arg; -> JitNode);

    jit_reexport!(getarg_c, reg: Reg, node: &JitNode);
    jit_reexport!(getarg_uc, reg: Reg, node: &JitNode);
    jit_reexport!(getarg_s, reg: Reg, node: &JitNode);
    jit_reexport!(getarg_us, reg: Reg, node: &JitNode);
    jit_reexport!(getarg_i, reg: Reg, node: &JitNode);
    jit_reexport!(getarg_ui, reg: Reg, node: &JitNode);
    jit_reexport!(getarg_l, reg: Reg, node: &JitNode);

    jit_reexport!(putargr, reg: Reg, arg: &JitNode);
    jit_reexport!(putargi, imm: JitWord, arg: &JitNode);

    jit_impl!(va_start, w);
    jit_impl!(va_arg, ww);
    jit_impl!(va_arg_d, ww);
    jit_impl!(va_end, w);

    jit_impl!(addr, www);
    jit_impl!(addi, i_www);
    jit_impl!(addcr, www);
    jit_impl!(addci, i_www);
    jit_impl!(addxr, www);
    jit_impl!(addxi, i_www);
    jit_impl!(subr, www);
    jit_impl!(subi, i_www);
    jit_impl!(subcr, www);
    jit_impl!(subci, i_www);
    jit_impl!(subxr, www);
    jit_impl!(subxi, i_www);

    //TODO: jit_rsbr
    jit_impl!(rsbi, i_www);

    jit_impl!(mulr, www);
    jit_impl!(muli, i_www);
    jit_impl!(qmulr, qww);
    jit_impl!(qmuli, i_qww);
    jit_impl!(qmulr_u, qww);
    jit_impl!(qmuli_u, i_qww);
    jit_impl!(divr, www);
    jit_impl!(divi, i_www);
    jit_impl!(divr_u, www);
    jit_impl!(divi_u, i_www);
    jit_impl!(qdivr, qww);
    jit_impl!(qdivi, i_qww);
    jit_impl!(qdivr_u, qww);
    jit_impl!(qdivi_u, i_qww);
    jit_impl!(remr, www);
    jit_impl!(remi, i_www);
    jit_impl!(remr_u, www);
    jit_impl!(remi_u, i_www);

    jit_impl!(andr, www);
    jit_impl!(andi, i_www);
    jit_impl!(orr, www);
    jit_impl!(ori, i_www);
    jit_impl!(xorr, www);
    jit_impl!(xori, i_www);

    jit_impl!(lshr, www);
    jit_impl!(lshi, i_www);
    jit_impl!(rshr, www);
    jit_impl!(rshi, i_www);
    jit_impl!(rshi_u, i_www);
    jit_impl!(rshr_u, www);

    jit_impl!(negr, ww);
    jit_impl!(comr, ww);

    jit_impl!(ltr, www);
    jit_impl!(lti, i_www);
    jit_impl!(ltr_u, www);
    jit_impl!(lti_u, i_www);
    jit_impl!(ler, www);
    jit_impl!(lei, i_www);
    jit_impl!(ler_u, www);
    jit_impl!(lei_u, i_www);
    jit_impl!(eqr, www);
    jit_impl!(eqi, i_www);
    jit_impl!(ger, www);
    jit_impl!(gei, i_www);
    jit_impl!(gtr, www);
    jit_impl!(gti, i_www);
    jit_impl!(gtr_u, www);
    jit_impl!(gti_u, i_www);
    jit_impl!(ner, www);
    jit_impl!(nei, i_www);

    jit_impl!(movr, ww);
    jit_impl!(movi, i_ww);

    jit_impl!(extr_c, ww);
    jit_impl!(extr_uc, ww);
    jit_impl!(extr_s, ww);
    jit_impl!(extr_us, ww);
    jit_impl!(extr_i, ww);
    jit_impl!(extr_ui, ww);

    jit_impl!(htonr_us, ww);
    jit_impl!(htonr_ui, ww);
    jit_impl!(htonr_ul, ww);

    jit_impl!(ldr_c, ww);
    jit_impl!(ldi_c, i_wp);
    jit_impl!(ldr_uc, ww);
    jit_impl!(ldi_uc, i_wp);
    jit_impl!(ldr_s, ww);
    jit_impl!(ldi_s, i_wp);
    jit_impl!(ldr_us, ww);
    jit_impl!(ldi_us, i_wp);
    jit_impl!(ldr_i, ww);
    jit_impl!(ldi_i, i_wp);
    jit_impl!(ldr_ui, ww);
    jit_impl!(ldi_ui, i_wp);
    jit_impl!(ldr_l, ww);
    jit_impl!(ldi_l, i_wp);

    jit_impl!(ldxr_c, www);
    jit_impl!(ldxi_c, i_www);
    jit_impl!(ldxr_uc, www);
    jit_impl!(ldxi_uc, i_www);
    jit_impl!(ldxr_s, www);
    jit_impl!(ldxi_us, i_www);
    jit_impl!(ldxr_i, www);
    jit_impl!(ldxi_i, i_www);
    jit_impl!(ldxr_ui, www);
    jit_impl!(ldxi_ui, i_www);
    jit_impl!(ldxr_l, www);
    jit_impl!(ldxi_l, i_www);

    jit_impl!(str_c, ww);
    jit_impl!(sti_c, i_wp);
    jit_impl!(str_s, ww);
    jit_impl!(sti_s, i_wp);
    jit_impl!(str_i, ww);
    jit_impl!(sti_i, i_wp);
    jit_impl!(str_l, ww);
    jit_impl!(sti_l, i_wp);

    jit_impl!(stxr_c, www);
    jit_impl!(stxi_c, i_www);
    jit_impl!(stxr_s, ww);
    jit_impl!(stxi_s, i_www);
    jit_impl!(stxr_i, ww);
    jit_impl!(stxi_i, i_www);
    jit_impl!(stxr_l, ww);
    jit_impl!(stxi_l, i_www);

    jit_branch!(bltr, r);
    jit_branch!(blti, i);
    jit_branch!(bltr_u, r);
    jit_branch!(blti_u, i);
    jit_branch!(bler, r);
    jit_branch!(blei, i);
    jit_branch!(beqr, r);
    jit_branch!(beqi, i);
    jit_branch!(bger, r);
    jit_branch!(bgei, i);
    jit_branch!(bger_u, r);
    jit_branch!(bgei_u, i);
    jit_branch!(bgtr, r);
    jit_branch!(bgti, i);
    jit_branch!(bgtr_u, r);
    jit_branch!(bgti_u, i);
    jit_branch!(bner, r);
    jit_branch!(bnei, i);
    jit_branch!(bmsr, r);
    jit_branch!(bmsi, i);
    jit_branch!(bmcr, r);
    jit_branch!(bmci, i);
    jit_branch!(boaddr, r);
    jit_branch!(boaddi, i);
    jit_branch!(boaddr_u, r);
    jit_branch!(boaddi_u, i);
    jit_branch!(bxaddr, r);
    jit_branch!(bxaddi, i);
    jit_branch!(bxaddr_u, r);
    jit_branch!(bxaddi_u, i);
    jit_branch!(bosubr, r);
    jit_branch!(bosubi, i);
    jit_branch!(bosubr_u, r);
    jit_branch!(bosubi_u, i);
    jit_branch!(bxsubr, r);
    jit_branch!(bxsubi, i);
    jit_branch!(bxsubr_u, r);
    jit_branch!(bxsubi_u, i);

    jit_impl!(jmpr, w);

    pub fn jmpi<'b>(&'b self) -> JitNode<'b> {
        // I looked at the lightning code, this will be copied
        JitNode{
            node: unsafe { jit_new_node!(p)(self.state, jit_code!(jmpi), std::ptr::null_mut::<c_void >()) },
            phantom: std::marker::PhantomData,
        }
    }

    jit_impl!(callr, w);
    jit_impl!(calli, i_p);

    jit_reexport!(prepare);
    jit_reexport!(pushargr, arg: Reg);
    jit_reexport!(pushargi, arg: JitWord);
    jit_reexport!(finishr, arg: Reg);
    jit_reexport!(finishi, arg: JitPointer; -> JitNode);
    jit_reexport!(ret);
    jit_reexport!(retr, rv: Reg);
    jit_reexport!(reti, rv: JitWord);
    jit_reexport!(retval_c, rv: Reg);
    jit_reexport!(retval_uc, rv: Reg);
    jit_reexport!(retval_s, rv: Reg);
    jit_reexport!(retval_us, rv: Reg);
    jit_reexport!(retval_i, rv: Reg);
    jit_reexport!(retval_ui, rv: Reg);
    jit_reexport!(retval_l, rv: Reg);
    jit_reexport!(epilog);


    //TODO float instructions

    jit_reexport!(address, node: &JitNode; -> JitPointer);

    jit_reexport!(patch, instr: &JitNode);
    jit_reexport!(patch_at, instr: &JitNode, target: &JitNode);
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
