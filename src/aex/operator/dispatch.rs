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

use super::Operator;
use aex::ast::Expr;
use aex::types::Type;
use aex::value::Value;

//// -----------------------------------------------------------------------------

//pub enum Dispatch {
//    Unary  ( UnaryDispatch),
//    Binary (BinaryDispatch),
//}
//
//impl fmt::Debug for Dispatch {
//    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
//        f.write_str(match *self {
//            Unary  (..) => "Unary",
//            Binary (..) => "Binary",
//        })
//    }
//}

// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Operand<'a> {
    pub val: Option<Value<'a>>,
    pub ty:  TypePtr<'a>,
}

impl<'a> Operand<'a> {
    pub fn is_const(&self) -> bool {
        match self.val {
            Some(ref v) => v.is_const(),
            None        => false,
        }
    }

    fn unwrap_const(self) -> Expr<'a> {
        match self.val {
            Some(v) => v.unwrap_const(),
            None    => panic!(),
        }
    }
}

pub type TypePtr<'a> = Cow<'a, Type<'a>>;

// -----------------------------------------------------------------------------
// Operator evaluation context

pub struct Context<'a> (&'a ());

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
            pub fn dispatch<'a>(&self, sel: Option<&str>,
                                $($arg: Operand<'a>),+, ctx: &mut Context<'a>)
                               -> Result<Operand<'a>, ()> {
                // Get implementation
                let op =
                    if let Some(s) = sel {
                        self.explicit_op(s)
                    } else if true $(&& $arg.is_const())+ {
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
    }
}

def_arity! {  UnaryOperator (a   ) :  UnaryOp, fail_unary  }
def_arity! { BinaryOperator (a, b) : BinaryOp, fail_binary }

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
use aex::operator::Arity::*;

static ADD: BinaryOperator = BinaryOperator {
    base:         Operator { chars: "+", prec: 5, assoc: Left, arity: Binary },
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

