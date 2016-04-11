// Expression Evaluation Helpers
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

use std::any::Any;
use std::fmt::{self, Display, Formatter};
use std::marker::PhantomData;
use num::BigInt;

use aex::ast::Expr;
use aex::codegen::CodeGenerator;
use aex::codegen::types::{ResolvedType, INT};
use aex::operator::Op;
use aex::pos::Pos;

//use aex::ast::*;
//use aex::scope::Scope;
//use aex::types::*;
//
//use super::Context;

// -----------------------------------------------------------------------------
// Eval - external interface to evaluator

pub trait Eval<'c> {
    fn eval(&self, expr: &Expr<'c>);
}

// -----------------------------------------------------------------------------
// Evaluator

pub struct Evaluator<'g, 'c: 'g, T> {
    cg: CodeGenerator<'g, 'c>,
    _t: PhantomData<T>,
}

impl<'g, 'c: 'g, T> Eval<'c> for Evaluator<'g, 'c, T> where T: Any {
    #[inline]
    #[allow(unused_must_use)]
    fn eval(&self, expr: &Expr<'c>) {
        // Delegate to the real `eval` and ignore its result.
        self.eval(expr);
    }
}

type V<'c, T> = Result<Value<'c, T>, ()>;


impl<'g, 'c: 'g, T> Evaluator<'g, 'c, T> where T: Any {
    fn eval(&self, expr: &Expr<'c>) -> V<'c, T> {
        match *expr {
            Expr::Ident  (pos, name)                  => self.eval_ident  (pos, name),
            Expr::Int    (pos, ref val)               => self.eval_int    (pos, val),
            Expr::Unary  (pos, op, sel, ref x)        => self.eval_unary  (pos, op, sel, x),
            Expr::Binary (pos, op, sel, ref x, ref y) => self.eval_binary (pos, op, sel, x, y),
            _ => panic!()
        }
    }

    fn eval_ident(&self, pos: Pos, name: &str) -> V<'c, T> {
        //Ok(Value {
        //    term: ?, 
        //    ty:   ?,
        //    pos:  pos
        //})
        panic!()
    }

    fn eval_int(&self, pos: Pos<'c>, val: &BigInt) -> V<'c, T> {
        Ok(Value {
            term: Self::t_from_const(val.clone()),
            ty:   INT,
            pos:  pos,
        })
    }

    fn eval_unary(&self,
                  pos: Pos<'c>,
                  op:  &Op,
                  sel: Option<&str>,
                  x:   &Expr<'c>)
                 -> V<'c, T> {
        let x = try!(self.eval(x));
        Self::do_unary(op, sel, x)
    }

    fn eval_binary(&self,
                   pos: Pos<'c>,
                   op:  &'c Op,
                   sel: Option<&'c str>,
                   x:   &Expr<'c>,
                   y:   &Expr<'c>)
                  -> V<'c, T> {
        let x = try!(self.eval(x));
        let y = try!(self.eval(y));
        let f = Any::downcast_ref::<DoBinary<T>>(&op.eval).unwrap();
        Ok(f(sel, x, y, pos))
    }

    // Target-specific
    fn t_from_const(val: BigInt) -> T {
        panic!()
    }

    // Target-specific
    fn do_unary(op:  &Op,
                sel: Option<&str>,
                x:   Value<'c, T>)
               -> V<'c, T> {
        panic!()
    }

    // Target-specific
    fn do_binary(op:  &Op,
                 sel: Option<&str>,
                 x:   Value<'c, T>,
                 y:   Value<'c, T>)
                -> V<'c, T> {
        panic!()
    }
}

type DoBinary<T> = &'static for<'a>
    Fn(Option<&str>, Value<'a, T>, Value<'a, T>, Pos<'a>) -> Value<'a, T>;

// -----------------------------------------------------------------------------
// Value - an evaluated expression

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Value<'c, T> {
    pub term: T,
    pub ty:   ResolvedType<'c>,
    pub pos:  Pos<'c>,
}

//impl<'c, T: Term<'c>> Display for Value<'c, T> {
//    #[inline(always)]
//    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//        Display::fmt(&self.term, f)
//    }
//}
//
//// -----------------------------------------------------------------------------
//// Term - a "term" (operand, location, etc.) in the target architecture
//
//pub trait Term<'c>: Display {
//    //// The Mode identifies what kind of term this is.
//    //type Mode;
//    //fn mode(&self) -> Self::Mode;
//
//    // A term must be able to hold a constant expression.
//    fn   is_const (&self)    -> bool;
//    fn   to_const (&self)    -> &Expr<'c>;
//    fn from_const (Expr<'c>) -> Self;
//}

