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
//use num::BigInt;

use aex::ast::Expr;
use aex::codegen::CodeGenerator;
use aex::codegen::typea::TypeA;
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

impl<'g, 'c: 'g, T> Eval<'c> for Evaluator<'g, 'c, T> {
    #[inline]
    #[allow(unused_must_use)]
    fn eval(&self, expr: &Expr<'c>) {
        // Delegate to the real `eval` and ignore its result.
        self.eval(expr);
    }
}

impl<'g, 'c: 'g, T> Evaluator<'g, 'c, T> {
    fn eval(&self, expr: &Expr<'c>) -> Result<Value<'c, T>, ()> {
        panic!();
    }
}

// -----------------------------------------------------------------------------
// Value - an evaluated expression

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Value<'c, T> {
    pub data: T,
    pub ty:   TypeA<'c>,
    pub pos:  Pos<'c>,
}

impl<'c, T: Display> Display for Value<'c, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.data, f)
    }
}

