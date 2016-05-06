// Freescale ColdFire Target
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

//mod loc;
//mod code_gen;

use std::marker::PhantomData;

use aex::ast::Expr;
use aex::operator::{Operator, OperatorTable, Const, Assoc, Fixity, binary_op};
//use aex::operator::Assoc::*;
//use aex::operator::Fixity::*;
//use aex::operator::Arity::*;
//use aex::operator::{Constness, Operand};
use aex::pos::{Source /*, Pos*/};
use aex::target::*;

pub struct ColdFire<'a> {
    _x: PhantomData<&'a ()>
}

impl<'a> ColdFire<'a> {
    pub fn new() -> Self {
        ColdFire { _x: PhantomData }
    }
}

impl<'a> Target for ColdFire<'a> {
    type String   = &'a str;
    type Source   = Source<'a>;
    type Operator = u8; //Operator<Value<'a>>;

    //type Term    = CfTerm<'a>;
    //type Expr    = Expr<'a, Self::Term>;
    //type Operand = Operand<'a, Self::Term>;

    //fn init_operators(&self, operators: &mut OperatorTable<Self::Term>) {
    //    operators.add(Operator::new("+", 7, Left, Infix, Binary(
    //        Box::new(|src, sel, args| Operand {
    //            term: CfTerm::B, kind: 42, src: src
    //        })
    //    )));
    //}
}

/*
type CfExpr<'a> = Expr<ColdFire<'a>>;

#[derive(Clone, Eq, PartialEq, Debug)]
enum Value<'a> {
    Const (Expr<ColdFire<'a>>),
    Other
}

impl<'a> Const for Value<'a> {
    type Expr = Expr<ColdFire<'a>>;

    #[inline]
    fn new_const(expr: Self::Expr) -> Self {
        Value::Const(expr)
    }

    #[inline]
    fn is_const(&self) -> bool { 
        match *self { Value::Const(_) => true, _ => false }
    }

    #[inline]
    fn unwrap_const(self) -> Self::Expr {
        match self { Value::Const(e) => e, _ => panic!() }
    }
}

fn def_operators<'a>(table: &mut OperatorTable<Value<'a>>) {
    table.add(binary_op::<Value<'a>>("q", 1, Assoc::Left, Fixity::Infix));
}
*/

//// Temporary
//pub enum CfTerm<'a> { A(&'a str), B }
//
//impl<'a> Constness for CfTerm<'a> {
//    type Expr = Expr<'a, Self>;
//
//    fn new_const(expr: Self::Expr) -> Self { panic!() }
//    fn is_const(&self) -> bool { panic!() }
//    fn to_const( self) -> Self::Expr { panic!() }
//}

