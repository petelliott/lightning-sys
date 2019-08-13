use crate::bindings;

pub enum Reg {
    R(bindings::jit_gpr_t),
    V(bindings::jit_gpr_t),
    F(bindings::jit_gpr_t),
}

pub struct JitNode<'a> {
    pub(crate) node:  *mut bindings::jit_node_t,
    pub(crate) phantom: std::marker::PhantomData<&'a ()>,
    //pub(crate) state: &'a JitState<'a>,
}

pub type JitWord = bindings::jit_word_t;
pub type JitUword = bindings::jit_uword_t;
pub type JitPointer = bindings::jit_pointer_t;


pub(crate) trait ToFFI {
    type Type;
    fn to_ffi(&self) -> Self::Type;
}

impl ToFFI for Reg {
    type Type = bindings::jit_gpr_t;

    //TODO: safe conversion
    fn to_ffi(&self) -> Self::Type {
        match self {
            Reg::R(i) => *i,
            Reg::V(i) => *i,
            Reg::F(i) => *i,
        }
    }
}

impl<'a> ToFFI for JitNode<'a> {
    type Type = *mut bindings::jit_node_t;

    fn to_ffi(&self) -> Self::Type {
        self.node
    }
}

pub(crate) trait FFISafe: Copy{}

// this is dumb, blame rust
impl FFISafe for JitPointer{}
impl FFISafe for i32{}
impl FFISafe for u32{}
impl FFISafe for i64{}
impl FFISafe for u64{}
impl FFISafe for f32{}
impl FFISafe for f64{}


impl<T: FFISafe> ToFFI for T {
    type Type = T;

    fn to_ffi(&self) -> Self::Type {
        *self
    }
}
