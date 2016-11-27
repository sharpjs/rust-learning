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

#![allow(unconditional_recursion)]

use std::ops::*;

use aex::ast::*;
use aex::context::Context;
use aex::message::Messages;
use aex::source::Source;
use aex::value::*;

use super::Operator;
use super::Assoc::*;
use super::dispatch::*;

static ADD: BinaryOperator = BinaryOperator {
    base:         Operator { chars: "+", prec: 5, assoc: Left },
    const_op:     None,
    implicit_op:  None,
    explicit_ops: &[]
};

const_op! { add(l, r) : ck_const_binary, tc::compat, Add::add, expr_add }

/// Constness check for a binary operation on const operands.
///
pub fn ck_const_unary<'x>(x:   &Operand<'x>,
                          log: &mut Messages<'x>)
                          ->   bool {
    if !x.is_const() {
        log.err_not_const(x.source());
        return false;
    }
    true
}

/// Constness check for a binary operation on const operands.
///
pub fn ck_const_binary<'a>(l:   &Operand<'a>,
                           r:   &Operand<'a>,
                           log: &mut Messages<'a>)
                           ->   bool {
    if !l.is_const() {
        log.err_not_const(l.source());
        return false;
    }
    if !r.is_const() {
        log.err_not_const(r.source());
        return false;
    }
    true
}

/// Constness check for a cast operation.
///
pub fn ck_const_cast<'a>(l:   &Operand<'a>,
                         r:   &Operand<'a>,
                         log: &mut Messages<'a>)
                         ->   bool {
    if !l.is_const() {
        log.err_not_const(l.source());
        return false;
    }
    if !r.is_type() {
        log.err_not_type(r.source());
        return false;
    }
    true
}

fn expr_add<'a>(lhs: Box<Expr<'a>>,
                rhs: Box<Expr<'a>>,
                src: Source<'a>)
               -> Expr<'a> {
    Expr::Binary(BinaryExpr {
      //op:  &ADD,
        sel: Id::default(),
        lhs: lhs,
        rhs: rhs,
        src: src
    })
}

// Temporary
mod tc {
    use aex::types::ResolvedType;
    pub fn compat<'a, T>(l: T, r: T) -> Option<ResolvedType<'a>> { None }
}

// Does eval typecheck?
pub fn stub_eval<'a>(ast: &Expr<'a>,
                     ctx: &mut Context<'a>)
                     ->   Result<Operand<'a>, ()> {
    match *ast {
        Expr::Binary(ref b) => {
            let l = try!(stub_eval(&b.lhs, ctx));
            let r = try!(stub_eval(&b.rhs, ctx));
            ADD.dispatch(None, l, r, ast, ctx)
        }
        _ => { Err(()) }
    }
}

mod tgt {
    use aex::ast::Expr;
    use aex::context::Context;
    use aex::operator::dispatch::OpcodeTable;
    use aex::types::res::ResolvedType;
    use aex::types::form::TypeForm;
    use aex::value::Operand;

    target_op! { add  (d, s) : CF_ADD,  32, d : check_values_2, check_types_2, check_forms_2 }
    target_op! { adda (d, s) : CF_ADDA, 32, d : check_values_2, check_types_2, check_forms_2 }

    static CF_ADD: OpcodeTable = &[
        (32, "add.l"),
    ];

    static CF_ADDA: OpcodeTable = &[
        (16, "adda.w"),
        (32, "adda.l"),
    ];

    fn check_values_2<'a>(l: &Operand<'a>,
                          r: &Operand<'a>)
                          -> bool {
        false
    }

    fn check_types_2<'a>(a: ResolvedType<'a>,
                         b: ResolvedType<'a>)
                         -> Option<ResolvedType<'a>> {
        None
    }
    
    fn check_forms_2(a:       TypeForm,
                     b:       TypeForm,
                     ret:     TypeForm,
                     default: u8)
                     ->       Option<u8> {
        Some(default)
    }
}

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

