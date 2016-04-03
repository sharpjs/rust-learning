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
// Evaluator

pub struct Evaluator<'g, 'c: 'g, L> {
    cg: CodeGenerator<'g, 'c>,
    _l: PhantomData<L>,
}

pub trait Eval<'c> {
    fn eval(&self, expr: &Expr<'c>);
}

impl<'g, 'c: 'g, L> Eval<'c> for Evaluator<'g, 'c, L> {
    #[inline]
    #[allow(unused_must_use)]
    fn eval(&self, expr: &Expr<'c>) {
        // Delegate to the real `eval` and ignore its result.
        self.eval(expr);
    }
}

impl<'g, 'c: 'g, L> Evaluator<'g, 'c, L> {
    fn eval(&self, expr: &Expr<'c>) -> Result<Value<'c, L>, ()> {
        panic!();
    }
}

// -----------------------------------------------------------------------------
// Value - an evaluated expression

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Value<'c, L> {
    pub loc: L,
    pub ty:  TypeA<'c>,
    pub pos: Pos<'c>,
}

impl<'c, L: Display> Display for Value<'c, L> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.loc, f)
    }
}

