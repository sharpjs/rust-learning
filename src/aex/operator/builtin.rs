// Built-In Operator Implementations
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

//use aex::ast::*;
//use aex::value::Value;

use super::Operator;
use super::Assoc::*;
use super::dispatch::*;

static ADD: BinaryOperator = BinaryOperator {
    base:         Operator { chars: "+", prec: 5, assoc: Left },
    const_op:     None,
    implicit_op:  None,
    explicit_ops: &[]
};

const_op! { add(l, r) : tc::compat }

// Temporary
mod tc {
    pub fn compat<T>(l: T, r: T) -> Option<()> { Some(()) }
}

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

