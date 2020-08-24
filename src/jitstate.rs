use crate::bindings;
use crate::Reg;
use crate::JitNode;
use crate::{JitWord, JitPointer};
use crate::ToFFI;
use std::ffi::CString;
use tt_call::*;

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

macro_rules! jit_reexport {
    ( $orig:ident, $fn:ident $(, $arg:ident : $typ:ty )*; -> JitNode<$life:lifetime>) => {
        pub fn $fn(&mut self $(, $arg: $typ )*) -> JitNode<$life> {
            JitNode{
                node: unsafe { bindings::$orig(self.state $(, $arg.to_ffi())*) },
                phantom: std::marker::PhantomData,
            }
        }
    };
    ( $orig:ident, $fn:ident $(, $arg:ident : $typ:ty )*; -> bool) => {
        pub fn $fn(&mut self $(, $arg: $typ )*) -> bool {
            unsafe { bindings::$orig(self.state $(, $arg.to_ffi())*) != 0 }
        }
    };
    ( $orig:ident, $fn:ident $(, $arg:ident : $typ:ty )*; -> $ret:ty) => {
        pub fn $fn(&mut self $(, $arg: $typ )*) -> $ret {
            unsafe { bindings::$orig(self.state $(, $arg.to_ffi())*) }
        }
    };
    ( $orig:ident, $fn:ident $(, $arg:ident : $typ:ty )*) => { jit_reexport!($orig, $fn $(, $arg : $typ)*; -> ()); }
}

macro_rules! jit_alias {
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

    jit_reexport!(_jit_emit, emit; -> JitPointer);

    jit_reexport!(_jit_address, address, node: &JitNode<'a>; -> JitPointer);

    jit_reexport!(_jit_forward_p, forward_p, node: &JitNode<'a>; -> bool);
    jit_reexport!(_jit_indirect_p, indirect_p, node: &JitNode<'a>; -> bool);
    jit_reexport!(_jit_target_p, target_p, node: &JitNode<'a>; -> bool);
    jit_reexport!(_jit_arg_register_p, arg_register_p, node: &JitNode<'a>; -> bool);
    jit_reexport!(_jit_callee_save_p, callee_save_p, reg: Reg; -> bool);
    jit_reexport!(_jit_pointer_p, pointer_p, ptr: JitPointer; -> bool);

    jit_reexport!(_jit_patch, patch, instr: &JitNode<'a>);
    jit_reexport!(_jit_patch_at, patch_at, instr: &JitNode<'a>, target: &JitNode<'a>);
    jit_reexport!(_jit_patch_abs, patch_abs, instr: &JitNode<'a>, target: JitPointer);
    jit_reexport!(_jit_realize, realize);

    // get_code needs argument mangling that jit_reexport currently does not
    // provide
    pub fn get_code(&self, code_size: Option<&mut JitWord>) -> JitPointer {
        unsafe { bindings::_jit_get_code(self.state, pointer_from(code_size)) }
    }

    jit_reexport!(_jit_set_code, set_code, buf: JitPointer, size: JitWord);

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

    jit_reexport!(_jit_set_data, set_data, buf: JitPointer, data_size: JitWord, flags: JitWord);

    jit_reexport!(_jit_print, print);
}

/// implementations of word-size-dependent aliases and exports
impl<'a> JitState<'a> {
    #[cfg(target_pointer_width = "64")]
    jit_alias!(getarg_l => getarg, reg: Reg, node: &JitNode<'a>);
    #[cfg(target_pointer_width = "32")]
    jit_alias!(getarg_i => getarg, reg: Reg, node: &JitNode<'a>);

    #[cfg(target_pointer_width = "32")]
    jit_alias!(ldr_i => ldr, targ: Reg, src: Reg; -> JitNode<'a>);
    #[cfg(target_pointer_width = "32")]
    jit_alias!(ldi_i => ldi, targ: Reg, src: JitPointer; -> JitNode<'a>);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(ldr_l => ldr, targ: Reg, src: Reg; -> JitNode<'a>);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(ldi_l => ldi, targ: Reg, src: JitPointer; -> JitNode<'a>);

    #[cfg(target_pointer_width = "32")]
    jit_alias!(ldxr_i => ldxr, targ: Reg, a: Reg, b: Reg; -> JitNode<'a>);
    #[cfg(target_pointer_width = "32")]
    jit_alias!(ldxi_i => ldxi, targ: Reg, src: Reg, off: JitWord; -> JitNode<'a>);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(ldxr_l => ldxr, targ: Reg, a: Reg, b: Reg; -> JitNode<'a>);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(ldxi_l => ldxi, targ: Reg, src: Reg, off: JitWord; -> JitNode<'a>);

    #[cfg(target_pointer_width = "32")]
    jit_alias!(str_i => str, targ: Reg, src: Reg; -> JitNode<'a>);
    #[cfg(target_pointer_width = "32")]
    jit_alias!(sti_i => sti, targ: JitPointer, src: Reg; -> JitNode<'a>);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(str_l => str, targ: Reg, src: Reg; -> JitNode<'a>);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(sti_i => sti, targ: JitPointer, src: Reg; -> JitNode<'a>);

    #[cfg(target_pointer_width = "32")]
    jit_alias!(stxr_i => stxr, off: Reg, targ: Reg, src: Reg; -> JitNode<'a>);
    #[cfg(target_pointer_width = "32")]
    jit_alias!(stxi_i => stxi, off: JitWord, targ: Reg, src: Reg; -> JitNode<'a>);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(stxr_l => stxr, off: Reg, targ: Reg, src: Reg; -> JitNode<'a>);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(stxi_l => stxi, off: JitWord, targ: Reg, src: Reg; -> JitNode<'a>);

    #[cfg(target_pointer_width = "32")]
    jit_alias!(retval_i => retval, rv: Reg);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(retval_l => retval, rv: Reg);

    #[cfg(target_pointer_width = "32")]
    jit_alias!(truncr_f_i => truncr_f, int: Reg, float: Reg; -> JitNode<'a>);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(truncr_f_l => truncr_f, int: Reg, float: Reg; -> JitNode<'a>);

    #[cfg(target_pointer_width = "32")]
    jit_alias!(truncr_d_i => truncr_d, int: Reg, float: Reg; -> JitNode<'a>);
    #[cfg(target_pointer_width = "64")]
    jit_alias!(truncr_d_l => truncr_d, int: Reg, float: Reg; -> JitNode<'a>);

    #[cfg(target_pointer_width = "64")]
    jit_reexport!(_jit_getarg_ui, getarg_ui, reg: Reg, node: &JitNode<'a>);
    #[cfg(target_pointer_width = "64")]
    jit_reexport!(_jit_getarg_l, getarg_l, reg: Reg, node: &JitNode<'a>);
    #[cfg(target_pointer_width = "64")]
    jit_reexport!(_jit_retval_ui, retval_ui, rv: Reg);
    #[cfg(target_pointer_width = "64")]
    jit_reexport!(_jit_retval_l, retval_l, rv: Reg);
}

/// implementations of general instructions
impl<'a> JitState<'a> {

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

    jit_reexport!(_jit_label, label; -> JitNode<'a>);
    jit_reexport!(_jit_forward, forward; -> JitNode<'a>);
    jit_reexport!(_jit_indirect, indirect; -> JitNode<'a>);
    jit_reexport!(_jit_link, link, node: &JitNode<'a>);

    jit_reexport!(_jit_prolog, prolog);
    jit_reexport!(_jit_ellipsis, ellipsis);

    jit_reexport!(_jit_allocai, allocai, size: i32; -> i32);
    jit_reexport!(_jit_allocar, allocar, off: Reg, size: Reg);

    jit_reexport!(_jit_arg, arg; -> JitNode<'a>);

    jit_reexport!(_jit_getarg_c, getarg_c, reg: Reg, node: &JitNode<'a>);
    jit_reexport!(_jit_getarg_uc, getarg_uc, reg: Reg, node: &JitNode<'a>);
    jit_reexport!(_jit_getarg_s, getarg_s, reg: Reg, node: &JitNode<'a>);
    jit_reexport!(_jit_getarg_us, getarg_us, reg: Reg, node: &JitNode<'a>);
    jit_reexport!(_jit_getarg_i, getarg_i, reg: Reg, node: &JitNode<'a>);

    jit_reexport!(_jit_putargr, putargr, reg: Reg, arg: &JitNode<'a>);
    jit_reexport!(_jit_putargi, putargi, imm: JitWord, arg: &JitNode<'a>);

    jit_reexport!(_jit_va_push, va_push, arg: Reg);

    pub fn rsbr(&mut self, a: Reg, b: Reg, c: Reg) -> JitNode<'a> {
        self.subr(a, c, b)
    }

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

    jit_reexport!(_jit_prepare, prepare);
    jit_reexport!(_jit_pushargr, pushargr, arg: Reg);
    jit_reexport!(_jit_pushargi, pushargi, arg: JitWord);
    jit_reexport!(_jit_finishr, finishr, arg: Reg);
    jit_reexport!(_jit_finishi, finishi, arg: JitPointer; -> JitNode<'a>);
    jit_reexport!(_jit_ret, ret);
    jit_reexport!(_jit_retr, retr, rv: Reg);
    jit_reexport!(_jit_reti, reti, rv: JitWord);
    jit_reexport!(_jit_retval_c, retval_c, rv: Reg);
    jit_reexport!(_jit_retval_uc, retval_uc, rv: Reg);
    jit_reexport!(_jit_retval_s, retval_s, rv: Reg);
    jit_reexport!(_jit_retval_us, retval_us, rv: Reg);
    jit_reexport!(_jit_retval_i, retval_i, rv: Reg);
    jit_reexport!(_jit_epilog, epilog);

    jit_reexport!(_jit_frame, frame, size: i32);
    jit_reexport!(_jit_tramp, tramp, fsize: i32);
}

/// implementations of 32-bit float instructions
impl<'a> JitState<'a> {
    jit_reexport!(_jit_arg_f, arg_f; -> JitNode<'a>);
    jit_reexport!(_jit_getarg_f, getarg_f, reg: Reg, arg: &JitNode<'a>);
    jit_reexport!(_jit_putargr_f, putargr_f, reg: Reg, arg: &JitNode<'a>);
    jit_reexport!(_jit_putargi_f, putargi_f, imm: f32, arg: &JitNode<'a>);

    pub fn rsbr_f(&mut self, a: Reg, b: Reg, c: Reg) -> JitNode<'a> {
        self.subr_f(a, c, b)
    }

    jit_reexport!(_jit_pushargr_f, pushargr_f, reg: Reg);
    jit_reexport!(_jit_pushargi_f, pushargi_f, imm: f32);
    jit_reexport!(_jit_retr_f, retr_f, reg: Reg);
    jit_reexport!(_jit_reti_f, reti_f, imm: f32);
    jit_reexport!(_jit_retval_f, retval_f, reg: Reg);
}

/// implementations of 64-bit float instructions
impl<'a> JitState<'a> {
    jit_reexport!(_jit_arg_d, arg_d; -> JitNode<'a>);
    jit_reexport!(_jit_getarg_d, getarg_d, reg: Reg, arg: &JitNode<'a>);
    jit_reexport!(_jit_putargr_d, putargr_d, reg: Reg, arg: &JitNode<'a>);
    jit_reexport!(_jit_putargi_d, putargi_d, imm: f64, arg: &JitNode<'a>);

    pub fn rsbr_d(&mut self, a: Reg, b: Reg, c: Reg) -> JitNode<'a> {
        self.subr_d(a, c, b)
    }

    jit_reexport!(_jit_pushargr_d, pushargr_d, reg: Reg);
    jit_reexport!(_jit_pushargi_d, pushargi_d, imm: f64);
    jit_reexport!(_jit_retr_d, retr_d, reg: Reg);
    jit_reexport!(_jit_reti_d, reti_d, imm: f64);
    jit_reexport!(_jit_retval_d, retval_d, reg: Reg);
}

macro_rules! private_make_func {
    {
        func = [{ $fname:ident $( < $( $life:lifetime ),+ > )? }]
        body = [{ $( $body:tt )* }]
        rettype = [{ $rettype:ty }]
        parmhead = [{ $( $parmhead:tt )* }]
        zipped = [{ $( $params:tt )* }]
    } => {
        pub fn $fname $( < $( $life ),+ > )? ( $( $parmhead )* $( $params )* ) -> $rettype {
            $( $body )*
        }
    };
}

/// Provides a compact way to call a tt-call macro from a pattern matched from
/// jit_entry.
macro_rules! mm {
    ( ( $entry:ident ( $( $inarg:ident ),* ) $root:ident ) => ( $( $types:ty ),* ) => ( $( $outarg:ident ),* ) ) => {
        make_func! {
            func = [{ $root }]
            body = [{
                unsafe {
                    self.$entry( $( $inarg.to_ffi().into() ),* )
                }
            }]
            rettype = [{ JitNode<'j> }]
            parmhead = [{ &mut self, }]
            parmnames = [{ $( $inarg ),* }]
            parmtypes = [{ $( $types ),* }]
        }
    };
}

/// Infer immediate type
macro_rules! it {
    ( _d ) => { f64 };
    ( _f ) => { f32 };
    (    ) => { JitWord };
}

macro_rules! jit_inner {
    // Ignores (by name) ---------------------------------------------------------------------------------------
    // Bottom-level new-node
    ( $a:tt [ new_node $(, $y:tt)* ] => $n:tt            $r:tt ) => { /* not part of the public interface */  };
    // Internal movs
    ( $a:tt [ mov, i, $k:tt, $w:tt ] => $n:tt            $r:tt ) => { /* not part of the public interface */  };
    ( $a:tt [ mov, r, $k:tt, $w:tt ] => $n:tt            $r:tt ) => { /* not part of the public interface */  };

    // Handlers (by name) --------------------------------------------------------------------------------------
    // Immediate calls and jumps
    ( $a:tt [ call, i              ] => $n:tt            $r:tt ) => { mm!{ $a => (JitPointer)         => $r } };
    ( $a:tt [ jmp, i               ] => $n:tt            $r:tt ) => { mm!{ $a => ()                   => $r } };
    // All ldi, sti, stxi (ldxi is handled by the jit_new_node_www catch-all)
    ( $a:tt [ ld, i    $(, $y:tt)? ] => $n:tt            $r:tt ) => { mm!{ $a => (Reg, JitPointer)    => $r } };
    ( $a:tt [ st, i    $(, $y:tt)? ] => $n:tt            $r:tt ) => { mm!{ $a => (JitPointer, Reg)    => $r } };
    ( $a:tt [ stx, i   $(, $y:tt)? ] => $n:tt            $r:tt ) => { mm!{ $a => (JitWord, Reg, Reg)  => $r } };
    // Movs
    ( $a:tt [ mov, i   $(, $y:tt)? ] => $n:tt            $r:tt ) => { mm!{ $a => (Reg, it!{$($y)?})   => $r } };
    // Varargs
    ( $a:tt [ va_arg $(, _d)?      ] => $n:tt            $r:tt ) => { mm!{ $a => (Reg, Reg)           => $r } };

    // Catch-alls (by signature) -------------------------------------------------------------------------------
    // All quad instructions
    ( $a:tt [ $q:tt, i $(, $y:tt)* ] => jit_new_node_qww $r:tt ) => { mm!{ $a => (i32, i32, Reg, Reg) => $r } };
    ( $a:tt [ $q:tt, r $(, $y:tt)* ] => jit_new_node_qww $r:tt ) => { mm!{ $a => (Reg, Reg, Reg, Reg) => $r } };
    // Branches
    ( $a:tt [ $q:tt    $(, $y:tt)* ] => jit_new_node_pwd $r:tt ) => { mm!{ $a => (Reg, f64)           => $r } };
    ( $a:tt [ $q:tt    $(, $y:tt)* ] => jit_new_node_pwf $r:tt ) => { mm!{ $a => (Reg, f32)           => $r } };
    ( $a:tt [ $q:tt, r $(, $y:tt)* ] => jit_new_node_pww $r:tt ) => { mm!{ $a => (Reg, Reg)           => $r } };
    ( $a:tt [ $q:tt, i $(, $y:tt)* ] => jit_new_node_pww $r:tt ) => { mm!{ $a => (Reg, JitWord)       => $r } };
    // All jit_new_node_ww[fd]
    ( $a:tt [ $q:tt, i, _d         ] => jit_new_node_wwd $r:tt ) => { mm!{ $a => (Reg, Reg, f64)      => $r } };
    ( $a:tt [ $q:tt, i, _f         ] => jit_new_node_wwf $r:tt ) => { mm!{ $a => (Reg, Reg, f32)      => $r } };
    // All jit_new_node_w+
    ( $a:tt [ $q:tt, r             ] => jit_new_node_w   $r:tt ) => { mm!{ $a => (Reg)                => $r } };
    ( $a:tt [ $q:tt                ] => jit_new_node_w   $r:tt ) => { mm!{ $a => (JitWord)            => $r } };
    ( $a:tt [ $q:tt, r $(, $y:tt)* ] => jit_new_node_ww  $r:tt ) => { mm!{ $a => (Reg, Reg)           => $r } };
    ( $a:tt [ $q:tt, i $(, $y:tt)? ] => jit_new_node_www $r:tt ) => { mm!{ $a => (Reg, Reg, JitWord)  => $r } };
    ( $a:tt [ $q:tt, r $(, $y:tt)? ] => jit_new_node_www $r:tt ) => { mm!{ $a => (Reg, Reg, Reg)      => $r } };

    // Fallbacks (generic patterns) ----------------------------------------------------------------------------
    (   ( $entry:ident ( $( $inarg:ident ),* ) $root:ident )
          [ $stem:ident $( , $suffix:ident )* ]
          => $invokes:ident( $enum:ident $( , $outarg:ident )* )
    ) => {
        // Ensure all patterns are caught explicitly before this
        compile_error!{ "Unhandled jit_entry -- jit_inner needs to be updated with a new pattern" }
    };

    ( $( $any:tt )* ) => { compile_error!{ "Unrecognized jit_entry -- check formatting of generated macros" } };
}

/// Defines an inherent method for `JitState` for each `jit_entry` that
/// corresponds to a `jit_new_node_*` call.
macro_rules! jit_filtered {
    {
        $caller:tt
        decl = [{ $entry:ident $inargs:tt }]
        root = [{ $root:ident }]
        parts = [{ $( $parts:ident )* }]
        invokes = [{ $invokes:ident $outargs:tt }]
    } => {
        jit_inner!{
            ( $entry $inargs $root )
              [ $( $parts ),* ]
              => $invokes $outargs
        }
    };
}

macro_rules! jit_entry_non_node {
    {
        $caller:tt
        decl = [{ $entry:ident( $( $inarg:ident ),* ) }]
        root = [{ destroy_state }]
        parts = [{ $stem:ident $( $suffix:ident )* }]
        invokes = [{ $invokes:ident( $( $outarg:ident ),* ) }]
    } => {
        make_func! {
            func = [{ $stem }]
            body = [{ unsafe { self.$entry( $( $inarg ),* ) } }]
            rettype = [{ () }]
            parmhead = [{ self, }] // Takes `self` by move, NOT by reference
            parmnames = [{ }]
            parmtypes = [{ }]
        }
    };
    { $( $tokens:tt )* } => {
        // Ignore these for now.
    };
}

include!(concat!(env!("OUT_DIR"), "/entries.rs"));

#[test]
fn trivial_invocation() {
    let mut entry_count = 0;

    trait MyDefault { fn default() -> Self; }

    impl MyDefault for f32        { fn default() -> Self { Default::default() } }
    impl MyDefault for f64        { fn default() -> Self { Default::default() } }

    #[cfg(target_pointer_width = "64")] /* avoid conflicting with JitWord */
    impl MyDefault for i32        { fn default() -> Self { Default::default() } }

    impl MyDefault for JitWord    { fn default() -> Self { Default::default() } }
    impl MyDefault for Reg        { fn default() -> Self { Reg::R(0)          } }
    impl MyDefault for JitPointer { fn default() -> Self { crate::types::NULL } }

    /// Calls the function represented by each `jit_entry` that corresponds to
    /// a `jit_new_node_*` call.
    macro_rules! mm {
        ( ( $entry:ident ( $( $inarg:ident ),* ) $root:ident ) => ( $( $types:ty ),* ) => ( $( $outarg:ident ),* ) ) => {
            {
                entry_count += 1;
                $( let $inarg = MyDefault::default(); )*
                let _ = $crate::Jit::new().new_state().$root( $( $inarg ),* );
            }
        };
    }

    /// Calls the function represented by each `jit_entry` that does *not*
    /// correspond to a `jit_new_node_*` call.
    macro_rules! jit_entry_non_node {
        {
            $caller:tt
            decl = [{ $entry:ident( $( $inarg:ident ),* ) }]
            root = [{ disassemble }]
            parts = [{ $stem:ident $( $suffix:ident )* }]
            invokes = [{ $invokes:ident( _jit $( , $outarg:ident )* ) }]
        } => {
            entry_count += 1;
            // Allow disassembly to be configured out.
            #[cfg(disassembly)]
            #[allow(unreachable_code)]
            #[allow(unused_variables)]
            {
                if false {
                    // We cannot yet actually invoke these, but at least we can
                    // check that the functions exist and take the right number
                    // of parameters.
                    $( let $outarg = unimplemented!(); )*
                    let _ = $crate::Jit::new().new_state().disassemble( $( $outarg ),* );
                }
            }
        };
        {
            $caller:tt
            decl = [{ $entry:ident( $( $inarg:ident ),* ) }]
            root = [{ $root:ident }]
            parts = [{ $stem:ident $( $suffix:ident )* }]
            invokes = [{ $invokes:ident( _jit $( , $outarg:ident )* ) }]
        } => {
            entry_count += 1;
            #[allow(unreachable_code)]
            #[allow(unused_variables)]
            {
                if false {
                    // We cannot yet actually invoke these, but at least we can
                    // check that the functions exist and take the right number
                    // of parameters.
                    $( let $outarg = unimplemented!(); )*
                    let _ = $crate::Jit::new().new_state().$root( $( $outarg ),* );
                }
            }
        };
        {
            $caller:tt
            decl = [{ $entry:ident( $( $inarg:ident ),* ) }]
            root = [{ $root:ident }]
            parts = [{ $stem:ident $( $suffix:ident )* }]
            invokes = [{ jit_cpu $( $other:tt )* }]
        } => {
            // Ignore macros that expand to an expression referencing
            // architecture-specific details.
        };
        {
            $caller:tt
            decl = [{ $entry:ident( $( $inarg:ident ),* ) }]
            root = [{ $root:ident }]
            parts = [{ $stem:ident $( $suffix:ident )* }]
            invokes = [{ $other:tt }]
        } => {
            // For now, ignore macros that expand to an expression that is not a
            // function call.
        };
        {
            $caller:tt
            decl = [{ $entry:ident( $( $inarg:ident ),* ) }]
            root = [{ $root:ident }]
            parts = [{ $stem:ident $( $suffix:ident )* }]
            invokes = [{ $( $other:tt )+ }]
        } => {
            entry_count += 1;
            // We cannot yet actually invoke these, but at least we can check
            // that the functions exist.
            let _ = JitState::$root;
        };
    }

    macro_rules! jit_entries {
        ( $( $tokens:tt )* ) => {
            { $( $tokens )* }
        };
    }

    include!{ concat!(env!("OUT_DIR"), "/entries.rs") }

    // The exact number of entry points depends on things like the target
    // architecture's word size, so we cannot robustly check for an exact
    // number, but we can put some useful bounds on the number that allow us to
    // catch egregious errors at least. We also do not necessarily want to break
    // immediately when a new version of GNU lightning adds or removes a few
    // entry points -- this is a sanity check only.
    assert!(entry_count > 400, "not enough entry points were seen");
    assert!(entry_count < 450, "too many entry points were seen");
}

