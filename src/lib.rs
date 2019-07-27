#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod bindings;

pub mod jit;
pub mod jitstate;

pub use jit::Jit;
pub use jitstate::JitState;

pub enum Reg {
    R(i32),
    V(i32),
    F(i32),
}
