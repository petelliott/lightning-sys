//! lightning-sys aims provides safe (as safe as a jit can be) rust bindings to GNU Lightning
//!
//! ## Examples:
//! ### a function that increments a number by one
//! ```
//! use lightning_sys::{Jit, Reg, JitPointer};
//!
//! let jit = Jit::new();
//! let mut js = jit.new_state();
//!
//! js.prolog();
//! let inarg = js.arg();
//! js.getarg_i(Reg::R(0), &inarg);
//! js.addi(Reg::R(0), Reg::R(0), 1);
//! js.retr(Reg::R(0));
//!
//! let incr = unsafe {
//!     js.emit::<extern fn(i32) -> i32>()
//! };
//!
//! assert_eq!(incr(5), 6);
//! assert_eq!(incr(6), 7);
//!
//! ```
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
