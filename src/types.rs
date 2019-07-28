use crate::bindings;
use crate::JitState;

pub enum Reg {
    R(i32),
    V(i32),
    F(i32),
}

pub struct JitNode<'a> {
    pub(crate) node:  *mut bindings::jit_node_t,
    pub(crate) state: &'a JitState<'a>,
}

pub type JitWord = bindings::jit_word_t;
pub type JitUword = bindings::jit_uword_t;
pub type JitPointer = bindings::jit_pointer_t;
