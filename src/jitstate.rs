use crate::bindings;
use crate::Reg;
use crate::JitNode;
use crate::{JitWord, JitPointer};
use crate::ToFFI;
use std::ffi::{CString, c_void};
use std::ptr::null_mut;

#[derive(Debug)]
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
    ( $op:ident, i_pw ) => { jit_impl_inner!($op, pw, a: JitPointer => _, b: Reg => JitWord); };
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

macro_rules! jit_store {
    ( $op:ident, ww ) => { jit_impl_inner!($op, ww, a: Reg => JitWord, b: Reg => JitWord); };

    ( $op:ident, i_pw ) => { jit_impl_inner!($op, pw, a: JitPointer => _, b: Reg => JitWord); };

    ( $op:ident, www ) => { jit_impl_inner!($op, www, a: Reg => JitWord, b: Reg => JitWord, c: Reg => JitWord); };

    ( $op:ident, i_www ) => { jit_impl_inner!($op, www, a: JitWord => _, b: Reg => JitWord, c: Reg => JitWord); };

}

macro_rules! jit_impl_type {
    ( $e:expr => _ ) => { $e };
    ( $e:expr => $t:ty ) => { $e as $t };
}

macro_rules! jit_impl_inner {
    ( $op:ident, $ifmt:ident $(, $arg:ident: $type:ty => $target:ty)* ) => {
        paste::item! {
            pub fn $op(&mut self $(, $arg: $type)*) -> JitNode<'a> {
                JitNode{
                    node: unsafe { bindings::[< _jit_new_node_ $ifmt >](self.state, bindings::jit_code_t::[< jit_code_ $op >] $(, jit_impl_type!($arg.to_ffi() => $target))*) },
                    phantom: std::marker::PhantomData,
                }
            }
        }
    };
    ( $op:ident, $ifmt:ident $(, $arg:ident: $type:ty)* ) => {
        jit_impl_inner!(kkk);
    };
}

macro_rules! jit_reexport {
    ( $fn:ident $(, $arg:ident : $typ:ty )*; -> JitNode) => {
        paste::item! {
            pub fn $fn(&mut self $(, $arg: $typ )*) -> JitNode<'a> {
                JitNode{
                    node: unsafe { bindings::[< _jit_ $fn >](self.state $(, $arg.to_ffi())*) },
                    phantom: std::marker::PhantomData,
                }
            }
        }
    };
    ( $fn:ident $(, $arg:ident : $typ:ty )*; -> bool) => {
        paste::item! {
            pub fn $fn(&mut self $(, $arg: $typ )*) -> bool {
                unsafe { bindings::[< _jit_ $fn >](self.state $(, $arg.to_ffi())*) != 0 }
            }
        }
    };
    ( $fn:ident $(, $arg:ident : $typ:ty )*; -> $ret:ty) => {
        paste::item! {
            pub fn $fn(&mut self $(, $arg: $typ )*) -> $ret {
                unsafe { bindings::[< _jit_ $fn >](self.state $(, $arg.to_ffi())*) }
            }
        }
    };
    ( $fn:ident $(, $arg:ident : $typ:ty )*) => { jit_reexport!($fn $(, $arg : $typ)*; -> ()); }
}

macro_rules! jit_imm {
    (i) => { JitWord };
    (r) => { Reg };
    (f) => { f32 };
    (d) => { f64 };
}

macro_rules! jit_branch {
    ( $fn:ident, $t:ident ) => {
        paste::item! {
            pub fn $fn(&mut self, a: Reg, b: jit_imm!($t)) -> JitNode<'a> {
                JitNode{
                    node: unsafe{ bindings::_jit_new_node_pww(self.state, bindings::jit_code_t::[< jit_code_ $fn >], null_mut::<c_void>(), a.to_ffi() as JitWord, b.to_ffi() as JitWord) },
                    phantom: std::marker::PhantomData,
                }
            }
        }
    };
}

macro_rules! jit_alias {
    ( $targ:ident => $new:ident $(, $arg:ident : $typ:ty )*; -> JitNode ) => {
        pub fn $new(&mut self $(, $arg: $typ )*) -> JitNode<'a> {
            self.$targ($( $arg ),*)
        }
    };
    ( $targ:ident => $new:ident $(, $arg:ident : $typ:ty )*; -> $ret:ty) => {
        pub fn $new(&mut self $(, $arg: $typ )*) -> $ret {
            self.$targ($( $arg ),*)
        }
    };
    ( $targ:ident => $new:ident $(, $arg:ident : $typ:ty )*) => { jit_alias!($targ => $new $(, $arg : $typ)*; -> ()); }
}

/// Convert a nullable reference into the C type representing it.
fn pointer_from<T>(p: Option<&mut T>) -> * mut T {
    p.map(|x| x as _).unwrap_or(std::ptr::null_mut())
}

/// `JitState` utility methods
impl<'a> JitState<'a> {
    pub fn clear_state(&mut self) {
        unsafe {
            bindings::_jit_clear_state(self.state);
        }
    }

    // there is no way to require a function type in a trait bound
    // without specifying the number of arguments
    pub unsafe fn cast_emit<T: Copy>(&mut self) -> T {
        *(&self.emit() as *const *mut core::ffi::c_void as *const T)
    }

    jit_reexport!(emit; -> JitPointer);

    jit_reexport!(address, node: &JitNode; -> JitPointer);

    jit_reexport!(forward_p, node: &JitNode; -> bool);
    jit_reexport!(indirect_p, node: &JitNode; -> bool);
    jit_reexport!(target_p, node: &JitNode; -> bool);
    jit_reexport!(arg_register_p, node: &JitNode; -> bool);
    jit_reexport!(callee_save_p, reg: Reg; -> bool);
    jit_reexport!(pointer_p, ptr: JitPointer; -> bool);

    jit_reexport!(patch, instr: &JitNode);
    jit_reexport!(patch_at, instr: &JitNode, target: &JitNode);
    jit_reexport!(patch_abs, instr: &JitNode, target: JitPointer);
    jit_reexport!(realize);

    // get_code needs argument mangling that jit_reexport currently does not
    // provide
    pub fn get_code(&self, code_size: Option<&mut JitWord>) -> JitPointer {
        unsafe { bindings::_jit_get_code(self.state, pointer_from(code_size)) }
    }

    jit_reexport!(set_code, buf: JitPointer, size: JitWord; -> ());

    // get_data needs argument mangling that jit_reexport currently does not
    // provide
    pub fn get_data(
        &self,
        data_size: Option<&mut JitWord>,
        note_size: Option<&mut JitWord>
    ) -> JitPointer {
        unsafe {
            bindings::_jit_get_data(
                self.state,
                pointer_from(data_size),
                pointer_from(note_size),
            )
        }
    }

    jit_reexport!(set_data, buf: JitPointer, data_size: JitWord, flags: JitWord; -> ());

    jit_reexport!(print);
}

/// implmentations of general instructions
impl<'a> JitState<'a> {
    jit_impl!(live, w);
    jit_impl!(align, w);

    pub fn name(&mut self, name: &str) -> JitNode<'a> {
        // I looked at the lightning code, this will be copied
        let cs = CString::new(name).unwrap();
        JitNode{
            node: unsafe { bindings::_jit_name(self.state, cs.as_ptr()) },
            phantom: std::marker::PhantomData,
        }
    }

    pub fn note(&mut self, file: Option<&str>, line: u32) -> JitNode<'a> {
        // I looked at the lightning code, this will be copied
        let cs = file
            .map(CString::new)
            .map(Result::unwrap)
            .map(|c| c.as_ptr())
            .unwrap_or(core::ptr::null());
        JitNode{
            node: unsafe { bindings::_jit_note(self.state, cs, line as i32) },
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
    #[cfg(target_pointer_width = "64")]
    jit_reexport!(getarg_ui, reg: Reg, node: &JitNode);
    #[cfg(target_pointer_width = "64")]
    jit_reexport!(getarg_l, reg: Reg, node: &JitNode);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(getarg_l => getarg, reg: Reg, node: &JitNode);
    #[cfg(target_pointer_width = "32")]
    jit_alias!(getarg_i => getarg, reg: Reg, node: &JitNode);

    jit_reexport!(putargr, reg: Reg, arg: &JitNode);
    jit_reexport!(putargi, imm: JitWord, arg: &JitNode);

    jit_impl!(va_start, w);
    jit_impl!(va_arg, ww);
    jit_impl!(va_arg_d, ww);
    jit_reexport!(va_push, arg: Reg);
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

    pub fn rsbr(&mut self, a: Reg, b: Reg, c: Reg) -> JitNode<'a> {
        self.subr(a, c, b)
    }

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
    jit_impl!(ger_u, www);
    jit_impl!(gei, i_www);
    jit_impl!(gei_u, i_www);
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
    #[cfg(target_pointer_width = "64")]
    jit_impl!(extr_i, ww);
    #[cfg(target_pointer_width = "64")]
    jit_impl!(extr_ui, ww);

    jit_impl!(htonr_us, ww);
    jit_alias!(htonr_us => ntohr_us, targ: Reg, src: Reg; -> JitNode);
    jit_impl!(htonr_ui, ww);
    jit_alias!(htonr_ui => ntohr_ui, targ: Reg, src: Reg; -> JitNode);
    #[cfg(target_pointer_width = "64")]
    jit_impl!(htonr_ul, ww);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(htonr_ul => ntohr_ul, targ: Reg, src: Reg; -> JitNode);
    #[cfg(target_pointer_width = "32")]
    jit_alias!(htonr_ui => htonr, targ: Reg, src: Reg; -> JitNode);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(htonr_ul => htonr, targ: Reg, src: Reg; -> JitNode);
    jit_alias!(htonr => ntohr, targ: Reg, src: Reg; -> JitNode);

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
    #[cfg(target_pointer_width = "64")]
    jit_impl!(ldr_ui, ww);
    #[cfg(target_pointer_width = "64")]
    jit_impl!(ldi_ui, i_wp);
    #[cfg(target_pointer_width = "64")]
    jit_impl!(ldr_l, ww);
    #[cfg(target_pointer_width = "64")]
    jit_impl!(ldi_l, i_wp);
    #[cfg(target_pointer_width = "32")]
    jit_alias!(ldr_i => ldr, targ: Reg, src: Reg; -> JitNode);
    #[cfg(target_pointer_width = "32")]
    jit_alias!(ldi_i => ldi, targ: Reg, src: JitPointer; -> JitNode);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(ldr_l => ldr, targ: Reg, src: Reg; -> JitNode);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(ldi_l => ldi, targ: Reg, src: JitPointer; -> JitNode);

    jit_impl!(ldxr_c, www);
    jit_impl!(ldxi_c, i_www);
    jit_impl!(ldxr_uc, www);
    jit_impl!(ldxi_uc, i_www);
    jit_impl!(ldxr_s, www);
    jit_impl!(ldxi_s, i_www);
    jit_impl!(ldxr_us, www);
    jit_impl!(ldxi_us, i_www);
    jit_impl!(ldxr_i, www);
    jit_impl!(ldxi_i, i_www);
    #[cfg(target_pointer_width = "64")]
    jit_impl!(ldxr_ui, www);
    #[cfg(target_pointer_width = "64")]
    jit_impl!(ldxi_ui, i_www);
    #[cfg(target_pointer_width = "64")]
    jit_impl!(ldxr_l, www);
    #[cfg(target_pointer_width = "64")]
    jit_impl!(ldxi_l, i_www);
    #[cfg(target_pointer_width = "32")]
    jit_alias!(ldxr_i => ldxr, targ: Reg, a: Reg, b: Reg; -> JitNode);
    #[cfg(target_pointer_width = "32")]
    jit_alias!(ldxi_i => ldxi, targ: Reg, src: Reg, off: JitWord; -> JitNode);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(ldxr_l => ldxr, targ: Reg, a: Reg, b: Reg; -> JitNode);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(ldxi_l => ldxi, targ: Reg, src: Reg, off: JitWord; -> JitNode);

    jit_store!(str_c, ww);
    jit_store!(sti_c, i_pw);
    jit_store!(str_s, ww);
    jit_store!(sti_s, i_pw);
    jit_store!(str_i, ww);
    jit_store!(sti_i, i_pw);
    #[cfg(target_pointer_width = "64")]
    jit_store!(str_l, ww);
    #[cfg(target_pointer_width = "64")]
    jit_store!(sti_l, i_pw);
    #[cfg(target_pointer_width = "32")]
    jit_alias!(str_i => str, targ: Reg, src: Reg; -> JitNode);
    #[cfg(target_pointer_width = "32")]
    jit_alias!(sti_i => sti, targ: JitPointer, src: Reg; -> JitNode);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(str_l => str, targ: Reg, src: Reg; -> JitNode);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(sti_i => sti, targ: JitPointer, src: Reg; -> JitNode);

    jit_store!(stxr_c, www);
    jit_store!(stxi_c, i_www);
    jit_store!(stxr_s, www);
    jit_store!(stxi_s, i_www);
    jit_store!(stxr_i, www);
    jit_store!(stxi_i, i_www);
    #[cfg(target_pointer_width = "64")]
    jit_store!(stxr_l, www);
    #[cfg(target_pointer_width = "64")]
    jit_store!(stxi_l, i_www);
    #[cfg(target_pointer_width = "32")]
    jit_alias!(stxr_i => stxr, off: Reg, targ: Reg, src: Reg; -> JitNode);
    #[cfg(target_pointer_width = "32")]
    jit_alias!(stxi_i => stxi, off: JitWord, targ: Reg, src: Reg; -> JitNode);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(stxr_l => stxr, off: Reg, targ: Reg, src: Reg; -> JitNode);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(stxi_l => stxi, off: JitWord, targ: Reg, src: Reg; -> JitNode);

    jit_branch!(bltr, r);
    jit_branch!(blti, i);
    jit_branch!(bltr_u, r);
    jit_branch!(blti_u, i);
    jit_branch!(bler, r);
    jit_branch!(blei, i);
    jit_branch!(bler_u, r);
    jit_branch!(blei_u, i);
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

    pub fn jmpi(&mut self) -> JitNode<'a> {
        // I looked at the lightning code, this will be copied
        JitNode{
            node: unsafe { bindings::_jit_new_node_p(self.state, bindings::jit_code_t::jit_code_jmpi, std::ptr::null_mut::<c_void >()) },
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

    pub fn get_note(
        &self,
        code: JitPointer,
        name: Option<&mut * mut std::os::raw::c_char>,
        file: Option<&mut * mut std::os::raw::c_char>,
        lineno: Option<&mut bindings::jit_int32_t>,
    ) -> bool {
        unsafe {
            bindings::_jit_get_note(
                self.state,
                code,
                pointer_from(name),
                pointer_from(file),
                pointer_from(lineno),
            ) != 0
        }
    }

    #[cfg(target_pointer_width = "64")]
    jit_reexport!(retval_ui, rv: Reg);
    #[cfg(target_pointer_width = "64")]
    jit_reexport!(retval_l, rv: Reg);
    #[cfg(target_pointer_width = "32")]
    jit_alias!(retval_i => retval, rv: Reg);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(retval_l => retval, rv: Reg);
    jit_reexport!(epilog);

    jit_reexport!(frame, size: i32);
    jit_reexport!(tramp, fsize: i32);
}

/// implmentations of 32-bit float instructions
impl<'a> JitState<'a> {
    jit_reexport!(arg_f; -> JitNode);
    jit_reexport!(getarg_f, reg: Reg, arg: &JitNode);
    jit_reexport!(putargr_f, reg: Reg, arg: &JitNode);
    jit_reexport!(putargi_f, imm: f32, arg: &JitNode);

    jit_impl!(addr_f, www);
    jit_impl!(addi_f, i_wwf);
    jit_impl!(subr_f, www);
    jit_impl!(subi_f, i_wwf);

    pub fn rsbr_f(&mut self, a: Reg, b: Reg, c: Reg) -> JitNode<'a> {
        self.subr_f(a, c, b)
    }

    jit_impl!(rsbi_f, i_wwf);
    jit_impl!(mulr_f, www);
    jit_impl!(muli_f, i_wwf);
    jit_impl!(divr_f, www);
    jit_impl!(divi_f, i_wwf);
    jit_impl!(negr_f, ww);
    jit_impl!(absr_f, ww);
    jit_impl!(sqrtr_f, ww);

    jit_impl!(ltr_f, www);
    jit_impl!(lti_f, i_wwf);
    jit_impl!(ler_f, www);
    jit_impl!(lei_f, i_wwf);
    jit_impl!(eqr_f, www);
    jit_impl!(eqi_f, i_wwf);
    jit_impl!(ger_f, www);
    jit_impl!(gei_f, i_wwf);
    jit_impl!(gtr_f, www);
    jit_impl!(gti_f, i_wwf);
    jit_impl!(ner_f, www);
    jit_impl!(nei_f, i_wwf);
    jit_impl!(unltr_f, www);
    jit_impl!(unlti_f, i_wwf);
    jit_impl!(unler_f, www);
    jit_impl!(unlei_f, i_wwf);
    jit_impl!(uneqr_f, www);
    jit_impl!(uneqi_f, i_wwf);
    jit_impl!(unger_f, www);
    jit_impl!(ungei_f, i_wwf);
    jit_impl!(ungtr_f, www);
    jit_impl!(ungti_f, i_wwf);
    jit_impl!(ltgtr_f, www);
    jit_impl!(ltgti_f, i_wwf);
    jit_impl!(ordr_f, www);
    jit_impl!(ordi_f, i_wwf);
    jit_impl!(unordr_f, www);
    jit_impl!(unordi_f, i_wwf);

    jit_impl!(truncr_f_i, ww);
    #[cfg(target_pointer_width = "64")]
    jit_impl!(truncr_f_l, ww);
    #[cfg(target_pointer_width = "32")]
    jit_alias!(truncr_f_i => truncr_f, int: Reg, float: Reg; -> JitNode);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(truncr_f_l => truncr_f, int: Reg, float: Reg; -> JitNode);

    jit_impl!(extr_f, ww);
    jit_impl!(extr_d_f, ww);
    jit_impl!(movr_f, ww);
    jit_impl!(movi_f, i_wf);

    jit_impl!(ldr_f, ww);
    jit_impl!(ldi_f, i_wp);
    jit_impl!(ldxr_f, www);
    jit_impl!(ldxi_f, i_www);

    jit_store!(str_f, ww);
    jit_store!(sti_f, i_pw);
    jit_store!(stxr_f, www);
    jit_store!(stxi_f, i_www);

    jit_branch!(bltr_f, r);
    jit_branch!(blti_f, f);
    jit_branch!(bler_f, r);
    jit_branch!(blei_f, f);
    jit_branch!(beqr_f, r);
    jit_branch!(beqi_f, f);
    jit_branch!(bger_f, r);
    jit_branch!(bgei_f, f);
    jit_branch!(bgtr_f, r);
    jit_branch!(bgti_f, f);
    jit_branch!(bner_f, r);
    jit_branch!(bnei_f, f);
    jit_branch!(bunltr_f, r);
    jit_branch!(bunlti_f, f);
    jit_branch!(bunler_f, r);
    jit_branch!(bunlei_f, f);
    jit_branch!(buneqr_f, r);
    jit_branch!(buneqi_f, f);
    jit_branch!(bunger_f, r);
    jit_branch!(bungei_f, f);
    jit_branch!(bungtr_f, r);
    jit_branch!(bungti_f, f);
    jit_branch!(bltgtr_f, r);
    jit_branch!(bltgti_f, f);
    jit_branch!(bordr_f, r);
    jit_branch!(bordi_f, f);
    jit_branch!(bunordr_f, r);
    jit_branch!(bunordi_f, f);

    jit_reexport!(pushargr_f, reg: Reg);
    jit_reexport!(pushargi_f, imm: f32);
    jit_reexport!(retr_f, reg: Reg);
    jit_reexport!(reti_f, imm: f32);
    jit_reexport!(retval_f, reg: Reg);
}

/// implmentations of 64-bit float instructions
impl<'a> JitState<'a> {
    jit_reexport!(arg_d; -> JitNode);
    jit_reexport!(getarg_d, reg: Reg, arg: &JitNode);
    jit_reexport!(putargr_d, reg: Reg, arg: &JitNode);
    jit_reexport!(putargi_d, imm: f64, arg: &JitNode);

    jit_impl!(addr_d, www);
    jit_impl!(addi_d, i_wwd);
    jit_impl!(subr_d, www);
    jit_impl!(subi_d, i_wwd);

    pub fn rsbr_d(&mut self, a: Reg, b: Reg, c: Reg) -> JitNode<'a> {
        self.subr_d(a, c, b)
    }

    jit_impl!(rsbi_d, i_wwd);
    jit_impl!(mulr_d, www);
    jit_impl!(muli_d, i_wwd);
    jit_impl!(divr_d, www);
    jit_impl!(divi_d, i_wwd);
    jit_impl!(negr_d, ww);
    jit_impl!(absr_d, ww);
    jit_impl!(sqrtr_d, ww);

    jit_impl!(ltr_d, www);
    jit_impl!(lti_d, i_wwd);
    jit_impl!(ler_d, www);
    jit_impl!(lei_d, i_wwd);
    jit_impl!(eqr_d, www);
    jit_impl!(eqi_d, i_wwd);
    jit_impl!(ger_d, www);
    jit_impl!(gei_d, i_wwd);
    jit_impl!(gtr_d, www);
    jit_impl!(gti_d, i_wwd);
    jit_impl!(ner_d, www);
    jit_impl!(nei_d, i_wwd);
    jit_impl!(unltr_d, www);
    jit_impl!(unlti_d, i_wwd);
    jit_impl!(unler_d, www);
    jit_impl!(unlei_d, i_wwd);
    jit_impl!(uneqr_d, www);
    jit_impl!(uneqi_d, i_wwd);
    jit_impl!(unger_d, www);
    jit_impl!(ungei_d, i_wwd);
    jit_impl!(ungtr_d, www);
    jit_impl!(ungti_d, i_wwd);
    jit_impl!(ltgtr_d, www);
    jit_impl!(ltgti_d, i_wwd);
    jit_impl!(ordr_d, www);
    jit_impl!(ordi_d, i_wwd);
    jit_impl!(unordr_d, www);
    jit_impl!(unordi_d, i_wwd);

    jit_impl!(truncr_d_i, ww);
    #[cfg(target_pointer_width = "64")]
    jit_impl!(truncr_d_l, ww);
    #[cfg(target_pointer_width = "32")]
    jit_alias!(truncr_d_i => truncr_d, int: Reg, float: Reg; -> JitNode);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(truncr_d_l => truncr_d, int: Reg, float: Reg; -> JitNode);

    jit_impl!(extr_d, ww);
    jit_impl!(extr_f_d, ww);
    jit_impl!(movr_d, ww);
    jit_impl!(movi_d, i_wd);

    jit_impl!(ldr_d, ww);
    jit_impl!(ldi_d, i_wp);
    jit_impl!(ldxr_d, www);
    jit_impl!(ldxi_d, i_www);

    jit_store!(str_d, ww);
    jit_store!(sti_d, i_pw);
    jit_store!(stxr_d, www);
    jit_store!(stxi_d, i_www);

    jit_branch!(bltr_d, r);
    jit_branch!(blti_d, d);
    jit_branch!(bler_d, r);
    jit_branch!(blei_d, d);
    jit_branch!(beqr_d, r);
    jit_branch!(beqi_d, d);
    jit_branch!(bger_d, r);
    jit_branch!(bgei_d, d);
    jit_branch!(bgtr_d, r);
    jit_branch!(bgti_d, d);
    jit_branch!(bner_d, r);
    jit_branch!(bnei_d, d);
    jit_branch!(bunltr_d, r);
    jit_branch!(bunlti_d, d);
    jit_branch!(bunler_d, r);
    jit_branch!(bunlei_d, d);
    jit_branch!(buneqr_d, r);
    jit_branch!(buneqi_d, d);
    jit_branch!(bunger_d, r);
    jit_branch!(bungei_d, d);
    jit_branch!(bungtr_d, r);
    jit_branch!(bungti_d, d);
    jit_branch!(bltgtr_d, r);
    jit_branch!(bltgti_d, d);
    jit_branch!(bordr_d, r);
    jit_branch!(bordi_d, d);
    jit_branch!(bunordr_d, r);
    jit_branch!(bunordi_d, d);

    jit_reexport!(pushargr_d, reg: Reg);
    jit_reexport!(pushargi_d, imm: f64);
    jit_reexport!(retr_d, reg: Reg);
    jit_reexport!(reti_d, imm: f64);
    jit_reexport!(retval_d, reg: Reg);
}
