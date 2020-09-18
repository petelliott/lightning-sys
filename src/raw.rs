//! Provides lowest-level `unsafe` bindings to the GNU lightning API, without type safety.
//!
//! This module implements the underpinnings of the high-level, type-safe API provided by the
//! crate. It may itself someday form part of the crate's public API, as well.
//!
//! This module implements `unsafe` entry points corresponding to the macro-based C API exposed by
//! GNU lightning. A direct translation of the API is not possible, since the public C API
//! explicitly assumes the presence of a lexical `_jit` state element in an un-hygienic way.
//!
//! The Rust APIs provided here look like this:
//! ```ignore
//! impl<'j> JitState<'j> {
//!     pub(crate) unsafe fn jit_absr_d(&mut self, u: jit_word_t, v: jit_word_t) -> JitNode<'j> {
//!         self.jit_new_node_ww(jit_code_t::jit_code_absr_d, u, v)
//!     }
//!
//!     pub(crate) unsafe fn jit_new_node_ww(
//!         &mut self,
//!         c: jit_code_t,
//!         u: jit_word_t,
//!         v: jit_word_t,
//!     ) -> JitNode<'j> {
//!         JitNode {
//!             node: _jit_new_node_ww(self.state, c, u, v),
//!             phantom: std::marker::PhantomData,
//!         }
//!     }
//! }
//! ```
//! corresponding to the original C macro definitions:
//! ```c
//! #define jit_absr_d(u,v)         jit_new_node_ww(jit_code_absr_d,u,v)
//! #define jit_new_node_ww(c,u,v)  _jit_new_node_ww(_jit,c,u,v)
//! ```

use crate::bindings::*;
use crate::JitNode;
use crate::jitstate::JitState;

use tt_call::*;

/// Given a `jit_new_node` macro name as an identifier, return the types of its
/// parameters. <sup>**[tt-call]**</sup>
///
/// Input: A `suffix` key containing an identifier starting with `jit_new_node`
/// Output: A `parmtypes` key containing a comma-separated list of types
macro_rules! jit_signature {
    { $c:tt suffix = [{ jit_new_node     }] } => { tt_return!{ $c parmtypes = [{                                                  }] } };
    { $c:tt suffix = [{ jit_new_node_d   }] } => { tt_return!{ $c parmtypes = [{ jit_float64_t                                    }] } };
    { $c:tt suffix = [{ jit_new_node_dp  }] } => { tt_return!{ $c parmtypes = [{ jit_float64_t, jit_pointer_t                     }] } };
    { $c:tt suffix = [{ jit_new_node_f   }] } => { tt_return!{ $c parmtypes = [{ jit_float32_t                                    }] } };
    { $c:tt suffix = [{ jit_new_node_fp  }] } => { tt_return!{ $c parmtypes = [{ jit_float32_t, jit_pointer_t                     }] } };
    { $c:tt suffix = [{ jit_new_node_p   }] } => { tt_return!{ $c parmtypes = [{ jit_pointer_t                                    }] } };
    { $c:tt suffix = [{ jit_new_node_pw  }] } => { tt_return!{ $c parmtypes = [{ jit_pointer_t, jit_word_t                        }] } };
    { $c:tt suffix = [{ jit_new_node_pwd }] } => { tt_return!{ $c parmtypes = [{ jit_pointer_t, jit_word_t, jit_float64_t         }] } };
    { $c:tt suffix = [{ jit_new_node_pwf }] } => { tt_return!{ $c parmtypes = [{ jit_pointer_t, jit_word_t, jit_float32_t         }] } };
    { $c:tt suffix = [{ jit_new_node_pww }] } => { tt_return!{ $c parmtypes = [{ jit_pointer_t, jit_word_t, jit_word_t            }] } };
    { $c:tt suffix = [{ jit_new_node_qww }] } => { tt_return!{ $c parmtypes = [{ jit_int32_t, jit_int32_t, jit_word_t, jit_word_t }] } };
    { $c:tt suffix = [{ jit_new_node_w   }] } => { tt_return!{ $c parmtypes = [{ jit_word_t                                       }] } };
    { $c:tt suffix = [{ jit_new_node_wd  }] } => { tt_return!{ $c parmtypes = [{ jit_word_t, jit_float64_t                        }] } };
    { $c:tt suffix = [{ jit_new_node_wf  }] } => { tt_return!{ $c parmtypes = [{ jit_word_t, jit_float32_t                        }] } };
    { $c:tt suffix = [{ jit_new_node_wp  }] } => { tt_return!{ $c parmtypes = [{ jit_word_t, jit_pointer_t                        }] } };
    { $c:tt suffix = [{ jit_new_node_ww  }] } => { tt_return!{ $c parmtypes = [{ jit_word_t, jit_word_t                           }] } };
    { $c:tt suffix = [{ jit_new_node_wwd }] } => { tt_return!{ $c parmtypes = [{ jit_word_t, jit_word_t, jit_float64_t            }] } };
    { $c:tt suffix = [{ jit_new_node_wwf }] } => { tt_return!{ $c parmtypes = [{ jit_word_t, jit_word_t, jit_float32_t            }] } };
    { $c:tt suffix = [{ jit_new_node_www }] } => { tt_return!{ $c parmtypes = [{ jit_word_t, jit_word_t, jit_word_t               }] } };
}

macro_rules! is_new_node_func {
    { $caller:tt input = [{ jit_new_node     }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };
    { $caller:tt input = [{ jit_new_node_d   }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };
    { $caller:tt input = [{ jit_new_node_dp  }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };
    { $caller:tt input = [{ jit_new_node_f   }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };
    { $caller:tt input = [{ jit_new_node_fp  }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };
    { $caller:tt input = [{ jit_new_node_p   }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };
    { $caller:tt input = [{ jit_new_node_pw  }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };
    { $caller:tt input = [{ jit_new_node_pwd }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };
    { $caller:tt input = [{ jit_new_node_pwf }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };
    { $caller:tt input = [{ jit_new_node_pww }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };
    { $caller:tt input = [{ jit_new_node_qww }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };
    { $caller:tt input = [{ jit_new_node_w   }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };
    { $caller:tt input = [{ jit_new_node_wd  }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };
    { $caller:tt input = [{ jit_new_node_wf  }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };
    { $caller:tt input = [{ jit_new_node_wp  }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };
    { $caller:tt input = [{ jit_new_node_ww  }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };
    { $caller:tt input = [{ jit_new_node_wwd }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };
    { $caller:tt input = [{ jit_new_node_wwf }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };
    { $caller:tt input = [{ jit_new_node_www }] } => { tt_return!{ $caller is_new_node_func = [{ true  }] } };

    { $caller:tt input = [{ $( $any:tt )*    }] } => { tt_return!{ $caller is_new_node_func = [{ false }] } };
}

/// Calls `zip_params` after dropping zero or more tokens from each list.
/// <sup>**[tt-call]**</sup>
///
/// Input: An `input` key containing three parenthesized lists:
///     1. A list of tokens, the length of which list determines how many to drop
///     1. A list of comma-separated identifiers to pass to `zip_params`
///     1. A list of comma-separated types to pass to `zip_params`
/// Output: An `input` key conforming to the expectations of `zip_params`
macro_rules! eat_params {
    {
        $caller:tt
        input = [{
            ( )
            ( $( $a_tail:ident ),* )
            ( $( $b_tail:ty    ),* )
        }]
    } => {
        zip_params! {
            $caller
            input = [{
                ( $( $a_tail ),* )
                ( $( $b_tail ),* )
                ( )
            }]
        }
    };
    {
        $caller:tt
        input = [{
            ( $eat:tt $( $rest:tt )* )
            ( $a_head:ident $( , $a_tail:ident )* )
            ( $b_head:ty    $( , $b_tail:ty    )* )
        }]
    } => {
        eat_params! {
            $caller
            input = [{
                ( $( $rest )* )
                ( $( $a_tail ),* )
                ( $( $b_tail ),* )
            }]
        }
    };
}

/// Zips a list of identifiers with a list of types into a parameter list.
/// <sup>**[tt-call]**</sup>
///
/// Input: An `input` key containing three parenthesized lists:
///     1. A list of comma-separated identifiers
///     1. A list of comma-separated types
///     1. A list of tokens to append to
/// Output: A `zipped` key containing a list of tokens that can be inserted
/// between parentheses to form a function's parameter list.
macro_rules! zip_params {
    {
        $caller:tt
        input = [{
            ( )
            ( )
            ( $( $zipped:tt )* )
        }]
    } => {
        tt_return! {
            $caller
            zipped = [{ $( $zipped )* }]
        }
    };
    {
        $caller:tt
        input = [{
            ( $a_head:ident $( , $a_tail:ident )* )
            ( $b_head:ty    $( , $b_tail:ty    )* )
            ( $( $zipped:tt )* )
        }]
    } => {
        zip_params! {
            $caller
            input = [{
                ( $( $a_tail ),* )
                ( $( $b_tail ),* )
                ( $( $zipped )* $a_head : $b_head, )
            }]
        }
    };
}

/// Defines an `pub(crate) unsafe fn` with a given name, parameters, and body.
/// <sup>**[tt-call]**</sup>
macro_rules! private_make_func {
    {
        func = [{ $fname:ident $( < $( $life:lifetime ),+ > )? }]
        body = [{ $( $body:tt )* }]
        rettype = [{ $rettype:ty }]
        parmhead = [{ $( $parmhead:tt )* }]
        zipped = [{ $( $params:tt )* }]
    } => {
        #[allow(clippy::missing_safety_doc)]
        #[allow(dead_code)] // Not all generated functions will be used.
        pub(crate) unsafe fn $fname $( < $( $life ),+ > )? ( $( $parmhead )* $( $params )* ) -> $rettype {
            $( $body )*
        }
    };
}

/// Defines a `pub(crate) unsafe fn` with a given name, parameters, and body.
/// <sup>**[tt-call]**</sup>
macro_rules! make_func {
    {
        func = [{ $fname:ident $( < $( $life:lifetime ),+ > )? }]
        body = [{ $( $body:tt )* }]
        rettype = [{ $rettype:ty }]
        parmhead = [{ $( $parmhead:tt )* }]
        $( parmskip = [{ $( $parmskip:tt )* }] )?
        parmnames = [{ $( $parmname:ident ),* }]
        parmtypes = [{ $( $parmtype:ty ),* }]
    } => {
        tt_call! {
            macro = [{ eat_params }]
            input = [{ ( $( $( $parmskip )* )? ) ( $( $parmname ),* ) ( $( $parmtype ),* ) }]
            ~~> private_make_func! {
                func = [{ $fname $( < $( $life ),+ > )? }]
                body = [{ $( $body )* }]
                rettype = [{ $rettype }]
                parmhead = [{ $( $parmhead )* }]
            }
        }
    };
}

/// Defines an inherent method for `JitState` for each `jit_entry` that
/// corresponds to a `jit_new_node_*` call.
macro_rules! jit_entry_for_node {
    {
        $caller:tt
        decl = [{ $entry:ident( $enum_in:ident $(, $inarg:ident )* ) }]
        root = [{ $root:ident }]
        parts = [{ new_node $( $suffix:ident )* }]
        invokes = [{ $invokes:ident( $jit:ident $( , $outarg:ident )* ) }]
    } => {
        tt_call! {
            macro = [{ jit_signature }]
            suffix = [{ $entry }]
            ~~> make_func! {
                func = [{ $entry }]
                body = [{
                    JitNode {
                        node: $invokes( self.state $( ,$outarg )* ),
                        phantom: std::marker::PhantomData,
                    }
                }]
                rettype = [{ JitNode<'j> }]
                parmhead = [{ &mut self, $enum_in: jit_code_t, }]
                parmnames = [{ $( $inarg ),* }]
            }
        }
    };
    {
        $caller:tt
        decl = [{ $entry:ident( $( $inarg:ident ),* ) }]
        root = [{ $root:ident }]
        parts = [{ $stem:ident $( $suffix:ident )* }]
        invokes = [{ $invokes:ident( $enum:ident, NULL $(, $outarg:ident )* ) }]
    } => {
        tt_call! {
            macro = [{ jit_signature }]
            suffix = [{ $invokes }]
            ~~> make_func! {
                func = [{ $entry }]
                body = [{ self.$invokes( jit_code_t::$enum, std::ptr::null_mut() $(, $outarg )* ) }]
                rettype = [{ JitNode<'j> }]
                parmhead = [{ &mut self, }]
                parmskip = [{ () }]
                parmnames = [{ _unused $(, $inarg )* }]
            }
        }
    };
    {
        $caller:tt
        decl = [{ $entry:ident( $( $inarg:ident ),* ) }]
        root = [{ $root:ident }]
        parts = [{ $stem:ident $( $suffix:ident )* }]
        invokes = [{ $invokes:ident( $enum:ident $( , $outarg:ident )* ) }]
    } => {
        tt_call! {
            macro = [{ jit_signature }]
            suffix = [{ $invokes }]
            ~~> make_func! {
                func = [{ $entry }]
                body = [{ self.$invokes( jit_code_t::$enum $( ,$outarg )* ) }]
                rettype = [{ JitNode<'j> }]
                parmhead = [{ &mut self, }]
                parmnames = [{ $( $inarg ),* }]
            }
        }
    };
}

macro_rules! jit_entry_non_node {
    { $( $tokens:tt )* } => {
        // Ignore these for now.
    };
}

macro_rules! jit_entry {
    {   $entry:ident $inargs:tt
            => $root:ident
            => [ $( $parts:ident ),* ]
            => $invokes:ident $outargs:tt
    } => {
        tt_if! {
            condition = [{ is_new_node_func }]
            input = [{ $invokes }]
            true = [{
                tt_call! {
                    macro = [{ jit_entry_for_node }]
                    decl = [{ $entry $inargs }]
                    root = [{ $root }]
                    parts = [{ $( $parts )* }]
                    invokes = [{ $invokes $outargs }]
                }
            }]
            false = [{
                tt_if! {
                    condition = [{ is_new_node_func }]
                    input = [{ $entry }]
                    true = [{
                        tt_call! {
                            macro = [{ jit_entry_for_node }]
                            decl = [{ $entry $inargs }]
                            root = [{ $root }]
                            parts = [{ $( $parts )* }]
                            invokes = [{ $invokes $outargs }]
                        }
                    }]
                    false = [{
                        tt_call! {
                            macro = [{ jit_entry_non_node }]
                            decl = [{ $entry $inargs }]
                            root = [{ $root }]
                            parts = [{ $( $parts )* }]
                            invokes = [{ $invokes $outargs }]
                        }
                    }]
                }
            }]
        }
    };
    {   $entry:ident $inargs:tt
            => $root:ident
            => [ $( $parts:ident ),* ]
            => $( $other:tt )*
    } => {
        tt_call! {
            macro = [{ jit_entry_non_node }]
            decl = [{ $entry $inargs }]
            root = [{ $root }]
            parts = [{ $( $parts )* }]
            invokes = [{ $( $other )* }]
        }
    };
}

macro_rules! jit_entries {
    ( $( $tokens:tt )* ) => {
        impl<'j> JitState<'j> {
            $( $tokens )*
        }
    };
}

include!(concat!(env!("OUT_DIR"), "/entries.rs"));

#[test]
#[allow(unreachable_code)]
#[allow(unused_variables)]
fn trivial_invocation() {
    let mut new_node_count = 0;
    let mut entry_count = 0;

    trait MyDefault { fn default() -> Self; }

    impl MyDefault for jit_word_t    { fn default() -> Self { Default::default() } }

    impl MyDefault for jit_float32_t { fn default() -> Self { Default::default() } }
    impl MyDefault for jit_float64_t { fn default() -> Self { Default::default() } }

    #[cfg(target_pointer_width = "64")] /* avoid conflicting with jit_word_t */
    impl MyDefault for jit_int32_t   { fn default() -> Self { Default::default() } }

    impl MyDefault for jit_pointer_t { fn default() -> Self { crate::types::NULL } }

    /// Calls the function represented by each `jit_entry` that corresponds to
    /// a `jit_new_node_*` call.
    macro_rules! jit_entry_for_node {
        {
            $caller:tt
            decl = [{ $entry:ident( $enum_in:ident $(, $inarg:ident )* ) }]
            root = [{ $root:ident }]
            parts = [{ new_node $( $suffix:ident )* }]
            invokes = [{ $invokes:ident( $enum:ident $( , $outarg:ident )* ) }]
        } => {
            new_node_count += 1;
            /* skip */
        };
        {
            $caller:tt
            decl = [{ $entry:ident( $( $inarg:ident ),* ) }]
            root = [{ $root:ident }]
            parts = [{ $stem:ident $( $suffix:ident )* }]
            invokes = [{ $invokes:ident( $enum:ident $( , $outarg:ident )* ) }]
        } => {
            {
                entry_count += 1;
                $( let $inarg = MyDefault::default(); )*
                let _ = $crate::Jit::new().new_state().$entry( $( $inarg ),* );
            }
        };
    }

    macro_rules! jit_entries {
        ( $( $tokens:tt )* ) => {
            unsafe { $( $tokens )* }
        };
    }

    include!{ concat!(env!("OUT_DIR"), "/entries.rs") }

    assert_eq!(new_node_count, 19, "an unexpected number of jit_new_node* entry points were seen");
    assert!(entry_count > 320, "an unexpected number of jit_new_node* callers were seen");
}

