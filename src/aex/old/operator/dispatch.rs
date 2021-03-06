// Operator Dispatch
//
// This file is part of AEx.
// Copyright (C) 2017 Jeffrey Sharp
//
// AEx is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published
// by the Free Software Foundation, either version 3 of the License,
// or (at your option) any later version.
//
// AEx is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See
// the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with AEx.  If not, see <http://www.gnu.org/licenses/>.

use std::fmt::{self, Debug, Formatter};

use super::Operator;

use aex::ast::Expr;
use aex::context::Context;
//use aex::source::Source;
//use aex::types::Type;
//use aex::util::bob::Bob;
use aex::value::*;

// -----------------------------------------------------------------------------
// Arity-specific operator types

macro_rules! def_arity {
    { $name:ident ( $($arg:ident),+ ) : $imp:ident, $fail:ident } => {

        // Type alias for operation implementation fn
        // :: operand * N -> context -> operand
        //
        pub type $imp = for<'a>
            fn ($($arg: Operand<'a>),+, ctx: &mut Context<'a>)
            -> Result<Operand<'a>, ()>;

        // An operator of a particular arity, with implementation fns.
        //
        pub struct $name {
            pub base:         Operator,
            pub const_op:     Option<$imp>,
            pub implicit_op:  Option<$imp>,
            pub explicit_ops: &'static [(&'static str, $imp)],
            // NB: We expect this list to be small, so linear search is OK.
            // It might be nice to have a static map here, but the current
            // solutions are a bit unweildy in stable Rust.
        }

        impl $name {
            // Select the appropriate implementation of the operator, and
            // invoke the implementation with the given operands and context.
            //
            pub fn dispatch<'a>(&self,
                                sel: Option<&str>,
                                $($arg: Operand<'a>),+,
                                ast: &Expr<'a>,
                                ctx: &mut Context<'a>)
                               -> Result<Operand<'a>, ()> {
                // Get implementation
                let op =
                    if let Some(s) = sel {
                        self.explicit_op(s)
                    } else if true $( && $arg.is_const() )+ {
                        self.const_op
                    } else {
                        self.implicit_op
                    };

                // Invoke implementation
                op.unwrap_or($fail)($($arg),+, ctx)
            }

            // Look up an implementation by its explicit selector.
            //
            fn explicit_op(&self, sel: &str) -> Option<$imp> {
                for &(s, f) in self.explicit_ops {
                    if s == sel { return Some(f) }
                }
                None
            }
        }

        // Built-in operator implementation that always fails.
        //
        pub fn $fail<'a>($($arg: Operand<'a>),+, ctx: &mut Context<'a>)
                        -> Result<Operand<'a>, ()> {
            panic!("Operator not supported by target") // TODO
        }

        // NB: Rust won't derive Clone, PartialEq, or Debug, because of the
        // higher-rank lifetimes (for<'a>) in this type.  It is a known issue:
        // (https://github.com/rust-lang/rust/issues/24000).  The workaround
        // is to implement the desired traits explicitly.

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.base == other.base
                && self  .const_op     .map(|f| f as *const ()) ==
                   other .const_op     .map(|f| f as *const ())
                && self  .implicit_op  .map(|f| f as *const ()) ==
                   other .implicit_op  .map(|f| f as *const ())
                && self  .explicit_ops .iter().map(|&(n, f)| (n, f as *const ())).eq(
                   other .explicit_ops .iter().map(|&(n, f)| (n, f as *const ()))   )
            }
        }

        impl Eq for $name { }

        impl Debug for $name {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                write!(f, "{}({:?})", stringify!($name), self.base)
            }
        }
    }
}

def_arity! {  UnaryOperator(a   ) :  UnaryImpl, fail_unary  }
def_arity! { BinaryOperator(a, b) : BinaryImpl, fail_binary }

// -----------------------------------------------------------------------------

#[macro_export]
macro_rules! const_op {
    { $name: ident ( $( $arg: ident ),+ )
        : $check_const: path
        , $check_types: path
        , $int_impl:    path
        , $expr_impl:   path
    } => {
        pub fn $name<'a>($( $arg:      Operand <'a> ),+   ,
                            ast:  &'a  Expr    <'a>       ,
                            ctx:  &mut Context <'a>       )
                            ->    Result<Operand<'a>, ()> {

            use $crate::aex::util::bob::{Bob as _Bob};

            // Constness check
            if !$check_const($( &$arg ),+, &mut ctx.out.log) {
                return Err(());
            }

            // Type check
            let ty = match $check_types($( &$arg.ty ),+) {
                Some(ty) => ty,
                None => {
                    ctx.out.log.err_incompatible_types($( $arg.source() )|+);
                    return Err(());
                }
            };

            // Try to reduce the operation
            let expr = {
                match ($( $arg.as_const() ),+,) {

                    // Integer literals
                    ($( &Expr::Int(ref $arg) ),+,) => {
                        use num::{BigInt as _BigInt, Zero as _Zero};
                        use $crate::aex::ast::{IntLit as _IntLit};

                        // Compute value
                        let mut val = _BigInt::zero();
                        $( val = $int_impl(&val, &$arg.val); )+

                        //// Value check
                        //if ty.contains(&n) == Some(false) {
                        //    ctx.out.log.err_value_out_of_range(pos);
                        //    return Err(());
                        //}

                        // Yield reduced value
                        Some(Expr::Int(_IntLit { val: val, src: ast.src() }))
                    },

                    // Not reducible
                    _ => { None }
                }
            };

            // Choose or build the result expression
            let (reduced, expr) = {
                if let Some(expr) = expr {

                    // Reducible operation
                    //   => use expression produced above
                    //
                    (true, _Bob::from(expr))

                } else if $( $arg.reduced )||+ {

                    // Irreducible operation on reduced operand(s)
                    //   => need new expression node to reference them
                    //
                    let expr = $expr_impl($( $arg.to_const() ),+, ast.src());
                    (true, _Bob::from(expr))

                } else {

                    // Irreducible operation on non-reduced operand(s)
                    //   => reuse expression from AST
                    //
                    (false, _Bob::from(ast))
                }
            };

            // Lift to operand
            Ok(Operand {
                value:   Value::Const(expr),
                ty:      ty,
                reduced: reduced,
            })
        }
    }
}

// -----------------------------------------------------------------------------

#[macro_export]
macro_rules! target_op {
    { $name:ident ( $( $arg:ident ),+ )
        : $opcodes:expr, $default:expr, $ret:ident
        : $mode_ck:expr, $type_ck:expr, $form_ck:expr
    } => {
        pub fn $name<'a>($( $arg:      Operand <'a> ),+   ,
                            ast:  &'a  Expr    <'a>       ,
                            ctx:  &mut Context <'a>       )
                            ->    Result<Operand<'a>, ()> {

            // Value/mode check
            //   - Does the op support these values / addressing modes?
            if !$mode_ck($( &$arg ),+) {
                ctx.out.log.err_no_op_for_addr_modes(ast.src());
                return Err(())
            }

            // Get type infos before we lose ownership of types
            let forms = ($( $arg.ty.info.form ),+,);

            // Type check
            // - Does the target op take operands of these types?
            // - What type should the result be?
            let ty = match $type_ck($( $arg.ty ),+) {
                Some(ty) => ty,
                None => {
                    ctx.out.log.err_incompatible_types(ast.src());
                    return Err(())
                }
            };

            // Pre-assemble result
            let ret = Operand {
                value:   $ret.value,
                ty:      ty,
                reduced: $ret.reduced,
            };

            // Unpack forms saved earlier
            let ($( $arg ),+,) = forms;

            // Form check
            // - Does the target op take operands of these storage widths?
            // - What is the width of the opcode that should be used?
            let width = match $form_ck($($arg),+, ret.ty.info.form, $default) {
                Some(w) => w,
                None => {
                    ctx.out.log.err_no_op_for_operand_types(ast.src());
                    return Err(())
                }
            };

            // Opcode lookup
            let op = match $crate::aex::operator::dispatch::select_opcode(width, $opcodes) {
                Some(op) => op,
                None     => {
                    ctx.out.log.err_no_op_for_operand_sizes(ast.src());
                    return Err(())
                }
            };

            //// Emit
            //ctx.out.asm.$write(op, $(&$n),+);

            // Cast result to checked type
            Ok(ret)
        }
    }
}

pub type OpcodeTable = &'static [(u8, &'static str)];

pub fn select_opcode(ty_width: u8, ops: OpcodeTable) -> Option<&'static str> {
    for &(op_width, op) in ops {
        if op_width == ty_width { return Some(op) }
    }
    None
}

