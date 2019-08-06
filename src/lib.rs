#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod bindings;

#[macro_use]
extern crate mashup;
extern crate num_traits;

pub mod jit;
pub use jit::Jit;

pub mod jitstate;
pub use jitstate::JitState;

pub mod types;
pub use types::Reg;
pub use types::JitNode;
pub use types::{JitWord, JitUword, JitPointer};
pub(crate) use types::ToFFI;
