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

pub struct Evaluator<'g, 'c: 'g, T: Term<'c>> {
    cg: CodeGenerator<'g, 'c>,
    _t: PhantomData<T>,
}

impl<'g, 'c: 'g, T: Term<'c>> Eval<'c> for Evaluator<'g, 'c, T> {
    #[inline]
    #[allow(unused_must_use)]
    fn eval(&self, expr: &Expr<'c>) {
        // Delegate to the real `eval` and ignore its result.
        self.eval(expr);
    }
}

type V<'c, T> = Result<Value<'c, T>, ()>;


impl<'g, 'c: 'g, T: Term<'c>> Evaluator<'g, 'c, T> {
    fn eval(&self, expr: &Expr<'c>) -> V<'c, T> {
        match *expr {
            Expr::Ident    (name)                  => self.eval_ident  (name),
            Expr::Int      (..)                    => self.eval_int    (expr),
            Expr::UnaryOp  (op, sel, ref x)        => self.eval_unary  (op, sel, x),
            Expr::BinaryOp (op, sel, ref x, ref y) => self.eval_binary (op, sel, x, y),
            _ => panic!()
        }
    }

    fn eval_ident(&self, name: &str) -> V<'c, T> {
        panic!()
    }

    fn eval_int(&self, val: &Expr<'c>) -> V<'c, T> {
        let pos = match *val { Expr::Int(pos, _) => pos, _ => panic!() };

        Ok(Value {
            term: T::from_const(val.clone()),
            ty:   INT,
            pos:  pos,
        })
    }

    fn eval_unary(&self,
                  op:  &'c Op,
                  sel: Option<&'c str>,
                  x:   &Expr<'c>)
                 -> V<'c, T> {
        let x = self.eval(x);
        panic!()
    }

    fn eval_binary(&self,
                   op:  &'c Op,
                   sel: Option<&'c str>,
                   x:   &Expr<'c>,
                   y:   &Expr<'c>)
                  -> V<'c, T> {
        let x = self.eval(x);
        let y = self.eval(y);
        panic!()
    }
}

// -----------------------------------------------------------------------------
// Value - an evaluated expression

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Value<'c, T: Term<'c>> {
    pub term: T,
    pub ty:   ResolvedType<'c>,
    pub pos:  Pos<'c>,
}

impl<'c, T: Term<'c>> Display for Value<'c, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.term, f)
    }
}

// -----------------------------------------------------------------------------
// Term - a "term" (operand, location, etc.) in the target architecture

pub trait Term<'c>: Display {
    // The Mode identifies what kind of term this is.
    type Mode;
    fn mode(&self) -> Self::Mode;

    // A term must be able to hold a constant expression.
    fn is_const(&self) -> bool;
    fn to_const(&self) -> &Expr<'c>;
    fn from_const(Expr<'c>) -> Self;
}

