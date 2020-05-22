use crate::bindings::*;
use crate::JitNode;
use crate::jitstate::JitState;

use tt_call::*;

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

macro_rules! private_make_func {
    {
        func = [{ $fname:ident $( < $( $life:lifetime ),+ > )? }]
        body = [{ $( $body:tt )* }]
        rettype = [{ $rettype:ty }]
        parmhead = [{ $( $parmhead:tt )* }]
        zipped = [{ $( $params:tt )* }]
    } => {
        #[allow(clippy::missing_safety_doc)]
        pub unsafe fn $fname $( < $( $life ),+ > )? ( $( $parmhead )* $( $params )* ) -> $rettype {
            $( $body )*
        }
    };
}

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

macro_rules! jit_entry {
    (   $entry:ident( $enum_in:ident $(, $inarg:ident )* )
          => $root:ident
          => [ new_node $( , $suffix:ident )* ]
          => $invokes:ident( $jit:ident $( , $outarg:ident )* )
    ) => {
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
    (   $entry:ident( $( $inarg:ident ),* )
          => $root:ident
          => [ $stem:ident $( , $suffix:ident )* ]
          => $invokes:ident( $enum:ident, NULL $(, $outarg:ident )* )
    ) => {
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
    (   $entry:ident( $( $inarg:ident ),* )
          => $root:ident
          => [ $stem:ident $( , $suffix:ident )* ]
          => $invokes:ident( $enum:ident $( , $outarg:ident )* )
    ) => {
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

macro_rules! jit_entries {
    ( $( $tokens:tt )* ) => {
        impl<'j> JitState<'j> {
            $( $tokens )*
        }
    };
}

include!(concat!(env!("OUT_DIR"), "/entries.rs"));

