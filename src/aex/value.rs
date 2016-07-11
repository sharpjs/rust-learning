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
use aex::target::*;

use self::Value::*;

// -----------------------------------------------------------------------------

/// A typed value -- the result of evaluating an expression.
///
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Value<'a> {
    /// Assembler constant
    Const(Expr<'a>),

    /// ColdFire value
    Cf(CfValue),
}

impl<'a> Value<'a> {
    pub fn is_const(&self) -> bool {
        match *self {
            Const(_) => true,
            _        => false,
        }
    }

    fn unwrap_const(self) -> Expr<'a> {
        match self {
            Const(e) => e,
            _        => panic!(),
        }
    }
}

