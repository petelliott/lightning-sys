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
//! let incr = unsafe { js.emit::<extern fn(i32) -> i32>() };
//! js.clear();
//!
//! assert_eq!(incr(5), 6);
//! assert_eq!(incr(6), 7);
//!
//! ```
//!
//! ### A simple function call to `printf`
//! ```
//! extern crate libc;
//!
//! use std::ffi::CString;
//! use lightning_sys::{Jit, JitWord, Reg, JitPointer};
//! use std::convert::TryInto;
//!
//! fn main() {
//!     let jit = Jit::new();
//!     let mut js = jit.new_state();
//!
//!     // make sure this outlives any calls
//!     let cs = CString::new("generated %d bytes\n").unwrap();
//!
//!     let start = js.note(file!(), line!());
//!     js.prolog();
//!     let inarg = js.arg();
//!     js.getarg_i(Reg::R(1), &inarg);
//!     js.prepare();
//!     js.pushargi(cs.as_ptr() as JitWord);
//!     js.ellipsis();
//!     js.pushargr(Reg::R(1));
//!     js.finishi(libc::printf as JitPointer);
//!     js.ret();
//!     js.epilog();
//!     let end = js.note(file!(), line!());
//!
//!     let my_function = unsafe{ js.emit::<extern fn(i32)>() };
//!     /* call the generated code, passing its size as argument */
//!     my_function((js.address(&end) as u64 - js.address(&start) as u64).try_into().unwrap());
//!     js.clear();
//!
//!     // TODO: dissasembly has not been implemented yet
//!     // js.dissasemble();
//! }
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
