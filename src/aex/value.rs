// Values
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

use aex::ast::Expr;
use aex::source::Source;
use aex::target::*;
use aex::types::ResolvedType;
use aex::util::bob::Bob;

use self::Value::*;

// Operand Structure Stack:
//
// Operand              + typed, possibly reduced
// Value<'a>            + other target-specific values
// Bob<'a, Expr<'a>>    + shared or owned
// Expr<'a>             + other kinds of expression
// FooExpr              specific attributes, source

// -----------------------------------------------------------------------------

/// A typed value -- the result of evaluating an expression.
///
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Value<'a> {
    /// Assembler constant
    Const(Bob<'a, Expr<'a>>),

    /// ColdFire value
    Cf(Bob<'a, CfValue<'a>>),

    /// Test value
    Test(Bob<'a, TestValue>),
}

impl<'a> Value<'a> {
    pub fn is_const(&self) -> bool {
        match *self {
            Const(_) => true,
            _        => false,
        }
    }

    pub fn as_const(&self) -> &Expr<'a> {
        match *self {
            Const(ref e) => &**e,
            _            => err_not_const(),
        }
    }

    pub fn unwrap_const(self) -> Bob<'a, Expr<'a>> {
        match self {
            Const(e) => e,
            _        => err_not_const(),
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Operand<'a> {
    pub val:     Option<Value<'a>>,
    pub ty:      ResolvedType<'a>,
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
            None        => err_not_const(),
        }
    }

    pub fn unwrap_const(self) -> Bob<'a, Expr<'a>> {
        match self.val {
            Some(v) => v.unwrap_const(),
            None    => err_not_const(),
        }
    }

    pub fn source(&self) -> Source<'a> {
        Source::BuiltIn // TODO
        //if let Some(val) = self.val {
        //}
    }
}

fn err_not_const() -> ! {
    panic!("Non-constant value given where constant is required.")
}

