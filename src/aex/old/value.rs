// Values
//
// This file is part of AEx.
// Copyright (C) 2017 Jeffrey Sharp
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

/// A typed value, the result of evaluating an expression.
///
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Operand<'a> {
    pub value:   Value<'a>,
    pub ty:      ResolvedType<'a>,
    pub reduced: bool,              // if obtained via reduction 
}

impl<'a> Operand<'a> {
    #[inline]
    pub fn is_const(&self) -> bool {
        self.value.is_const()
    }

    #[inline]
    pub fn is_type(&self) -> bool {
        self.value.is_type()
    }

    #[inline]
    pub fn as_const(&self) -> &Expr<'a> {
        self.value.as_const()
    }

    #[inline]
    pub fn to_const(self) -> Box<Expr<'a>> {
        self.value.to_const()
    }

    pub fn source(&self) -> Source<'a> {
        Source::BuiltIn // TODO
        //if let Some(val) = self.val {
        //}
    }
}

// -----------------------------------------------------------------------------

/// Sum type of all constant and target-specific value types.
///
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Value<'a> {
    /// Type constant
    Type, // actual type stored in containing Operand

    /// Assembler constant
    Const(Bob<'a, Expr<'a>>),

    /// ColdFire value
    Cf(Bob<'a, CfValue<'a>>),

    /// Test value
    Test(Bob<'a, TestValue>),
}

impl<'a> Value<'a> {
    #[inline]
    pub fn is_type(&self) -> bool {
        *self == Value::Type
    }

    #[inline]
    pub fn is_const(&self) -> bool {
        match *self {
            Const(_) => true,
            _        => false,
        }
    }

    #[inline]
    pub fn as_const(&self) -> &Expr<'a> {
        match *self {
            Const(ref e) => &**e,
            _            => err_not_const(),
        }
    }

    #[inline]
    pub fn to_const(self) -> Box<Expr<'a>> {
        match self {
            Const(e) => e.into_box(),
            _        => err_not_const(),
        }
    }
}

// -----------------------------------------------------------------------------

fn err_not_const() -> ! {
    panic!("Non-constant value given where constant is required.")
}

