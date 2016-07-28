// Operator Dispatch
//
// This file is part of AEx.
// Copyright (C) 2016 Jeffrey Sharp
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

use std::borrow::Cow;
use std::fmt::{self, Debug, Formatter};

use super::Operator;
use aex::ast::Expr;
use aex::context::Context;
use aex::source::Source;
use aex::types::Type;
use aex::value::Value;

// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Operand<'a> {
    pub val:     Option<Value<'a>>,
    pub ty:      TypePtr<'a>,
    pub reduced: bool, // if value is the reduction of some other expression
}

impl<'a> Operand<'a> {
    pub fn is_const(&self) -> bool {
        match self.val {
            Some(ref v) => v.is_const(),
            None        => false,
        }
    }

    pub fn as_const(&self) -> &Expr<'a> {
        match self.val {
            Some(ref v) => v.as_const(),
            None        => panic!("Non-constant operand given where constant is required."),
        }
    }

    pub fn source(&self) -> Source<'a> {
        Source::BuiltIn // TODO
        //if let Some(val) = self.val {
        //}
    }
}

pub type TypePtr<'a> = Cow<'a, Type<'a>>;

pub type OpResult<'a> = Result<Operand<'a>, ()>;

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
                                //orig: &BinaryExpr<'a>
                                sel: Option<&str>,
                                $($arg: Operand<'a>),+, ctx: &mut Context<'a>)
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
    { $name:ident($($arg:ident),+)
        : $check_types:path, $int_impl:path, $expr_impl:path
    } => {
        //use aex::ast::Expr;

        pub fn $name<'a>($( $arg: Operand<'a> ),+,
                         orig: &BinaryExpr<'a>,
                         ctx:  &mut Context<'a>)
                        -> OpResult<'a> {

            // Constness check
            if $( !$arg.is_const() )|+ {
                panic!("Constant operation invoked for non-constant operand.")
            }

            // Type check
            let ty = match $check_types($( &$arg.ty ),+) {
                Some(ty) => ty,
                None     => {
                    ctx.out.log.err_incompatible_types($( $arg.source() )|+);
                    return Err(());
                }
            };

            // Evaluate
            let expr = match ($( $arg.as_const() ),+,) {
                // Integer literals
                ($( &Expr::Int(ref $arg) ),+,) => {
                    use num::{BigInt, Zero};
                    use aex::ast::IntLit;

                    // Compute value
                    let mut val = BigInt::zero();
                    $( val = $int_impl(&val, &$arg.val); )+

                    //// Value check
                    //if ty.contains(&n) == Some(false) {
                    //    ctx.out.log.err_value_out_of_range(pos);
                    //    return Err(());
                    //}

                    // Yield reduced expression
                    Expr::Int(IntLit { val: val, src: orig.src })
                },

                // Opaque expressions
                ($( $arg ),+,) => {
                    // Must make a new expression object, as evaluation might
                    // have reduced some child nodes.
                    $expr_impl($( $arg ),+, orig.src)
                }
            };

            // Cast to checked type
            Ok(Operand {
                val: Some(Value::Const(expr)),
                ty: ty,
                reduced: true,
            })
        }
    }
}

// -----------------------------------------------------------------------------

//macro_rules! op {
//    { $name:ident ( $($arg:ident),+ )
//        : $opcodes:expr, $default:expr, $ret:ident
//        : $mode_ck:expr, $type_ck:expr, $form_ck:expr
//    } => {
//        pub fn $name<'a>($($arg: Operand<'a>),+, ctx: Context<'a>)
//                           -> Result<Operand<'a>, ()> {
//            // Value/mode check
//            // - Does the target op support these values / addressing modes?
//            if !$mode_ck($(&$arg.value),+) {
//                ctx.out.log.err_no_op_for_addr_modes(Pos::bof("a"));
//                return Err(())
//            }
//
//            // Get forms before we lose ownership of types
//            let forms = ($($arg.ty.form()),+,);
//
//            // Type check
//            // - Does the target op take operands of these types?
//            // - What type should the result be?
//            let ty = match $type_ck($($arg.ty),+) {
//                Some(ty) => ty,
//                None => {
//                    ctx.out.log.err_incompatible_types(Pos::bof("a"));
//                    return Err(())
//                }
//            };
//
//            // Pre-assemble result
//            let ret = Operand { value: $ret.value, ty: ty, source: $ret.source };
//
//            // Unpack forms saved earlier
//            let ($($arg),+,) = forms;
//
//            // Form check
//            // - Does the target op take operands of these storage widths?
//            // - What is the width of the opcode that should be used?
//            let width = match $form_ck($($arg),+, ret.ty.form(), $default) {
//                Some(w) => w,
//                None => {
//                    ctx.out.log.err_no_op_for_operand_types(Pos::bof("a"));
//                    return Err(())
//                }
//            };
//
//            // Opcode lookup
//            let op = match select_opcode(width, $opcodes) {
//                Some(op) => op,
//                None     => {
//                    ctx.out.log.err_no_op_for_operand_sizes(Pos::bof("a"));
//                    return Err(())
//                }
//            };
//
//            //// Emit
//            //ctx.out.asm.$write(op, $(&$n),+);
//
//            // Cast result to checked type
//            Ok(ret)
//        }
//    }
//}
//
//pub type OpcodeTable = &'static [(u8, &'static str)];
//
//fn select_opcode(ty_width: u8, ops: OpcodeTable) -> Option<&'static str> {
//    for &(op_width, op) in ops {
//        if op_width == ty_width { return Some(op) }
//    }
//    None
//}

// -----------------------------------------------------------------------------

use aex::operator::Assoc::*;

static ADD: BinaryOperator = BinaryOperator {
    base:         Operator { chars: "+", prec: 5, assoc: Left },
    const_op:     None,
    implicit_op:  None,
    explicit_ops: &[]
};

//op! { add  (d, s) : ADD,  32, d : check_values_2, check_types_2, check_forms_2 }
//op! { adda (d, s) : ADDA, 32, d : check_values_2, check_types_2, check_forms_2 }
//
//static ADD: OpcodeTable = &[
//    (32, "add.l"),
//];
//
//static ADDA: OpcodeTable = &[
//    (16, "adda.w"),
//    (32, "adda.l"),
//];
//
//fn check_values_2(a: &V, b: &V) -> bool {
//    panic!()
//}
//
//fn check_types_2<'a>(a: TypePtr<'a>, b: TypePtr<'a>) -> Option<TypePtr<'a>> {
//    panic!()
//}
//
//fn check_forms_2(a: TypeForm, b: TypeForm,
//                 ret: TypeForm,
//                 default: u8)
//                -> Option<u8> {
//    Some(default)
//}
//
//// -----------------------------------------------------------------------------
//
//pub fn def_builtin_operators<V: Const>(table: &mut OperatorTable) {
//    for op in builtin_operators() { table.add(op) }
//}
//
//fn builtin_operators<'a: Const>() -> Vec<Operator> {
//    vec![
//        // Postfix Unary
//         unary_op( "++" , 10 , Left  , Postfix ),
//         unary_op( "--" , 10 , Left  , Postfix ),
//        // Prefix Unary
//         unary_op( "!"  ,  9 , Right , Prefix  ),
//         unary_op( "~"  ,  9 , Right , Prefix  ),
//         unary_op( "-"  ,  9 , Right , Prefix  ),
//         unary_op( "+"  ,  9 , Right , Prefix  ),
//         unary_op( "&"  ,  9 , Right , Prefix  ),
//        // Multiplicative
//        binary_op( "*"  ,  8 , Left  , Infix   ),
//        binary_op( "/"  ,  8 , Left  , Infix   ),
//        binary_op( "%"  ,  8 , Left  , Infix   ),
//        // Additive                          
//        binary_op( "+"  ,  7 , Left  , Infix   ),
//        binary_op( "-"  ,  7 , Left  , Infix   ),
//        // Bitwise Shift                      
//        binary_op( "<<" ,  6 , Left  , Infix   ),
//        binary_op( ">>" ,  6 , Left  , Infix   ),
//        // Bitwise Boolean                   
//        binary_op( "&"  ,  5 , Left  , Infix   ),
//        binary_op( "^"  ,  4 , Left  , Infix   ),
//        binary_op( "|"  ,  3 , Left  , Infix   ),
//        // Bitwise Manipulation              
//        binary_op( ".~" ,  2 , Left  , Infix   ),
//        binary_op( ".!" ,  2 , Left  , Infix   ),
//        binary_op( ".+" ,  2 , Left  , Infix   ),
//        binary_op( ".?" ,  2 , Left  , Infix   ),
//        // Comparison                     
//        binary_op( "?"  ,  1 , Left  , Postfix ),
//        binary_op( "<>" ,  1 , Left  , Infix   ),
//        binary_op( "==" ,  1 , Left  , Infix   ),
//        binary_op( "!=" ,  1 , Left  , Infix   ),
//        binary_op( "<"  ,  1 , Left  , Infix   ),
//        binary_op( "<=" ,  1 , Left  , Infix   ),
//        binary_op( ">"  ,  1 , Left  , Infix   ),
//        binary_op( ">=" ,  1 , Left  , Infix   ),
//        // Assignment                                              
//        binary_op( "="  ,  0 , Right , Infix   ),
//    ]
//}

// This needs to go somewhere:
//
//Expr::Negate     (ref e, None)        => write!(f, "-{}", e),
//Expr::Complement (ref e, None)        => write!(f, "~{}", e),
//Expr::Multiply   (ref l, ref r, None) => write!(f, "({} * {})",  l, r),
//Expr::Divide     (ref l, ref r, None) => write!(f, "({} / {})",  l, r),
//Expr::Modulo     (ref l, ref r, None) => write!(f, "({} % {})",  l, r),
//Expr::Add        (ref l, ref r, None) => write!(f, "({} + {})",  l, r),
//Expr::Subtract   (ref l, ref r, None) => write!(f, "({} - {})",  l, r),
//Expr::ShiftL     (ref l, ref r, None) => write!(f, "({} << {})", l, r),
//Expr::ShiftR     (ref l, ref r, None) => write!(f, "({} >> {})", l, r),
//Expr::BitAnd     (ref l, ref r, None) => write!(f, "({} & {})",  l, r),
//Expr::BitXor     (ref l, ref r, None) => write!(f, "({} ^ {})",  l, r),
//Expr::BitOr      (ref l, ref r, None) => write!(f, "({} | {})",  l, r),

