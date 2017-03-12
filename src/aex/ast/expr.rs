// AST: Expressions
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

use std::fmt::{self, Display, Formatter};
use super::*;
//use aex::asm::{AsmDisplay, AsmStyle};

/// An expression.
#[derive(Clone, Debug)]
pub enum Expr<'a, C = ()> {
    /// Identifier
    Id(Id<'a, C>),

    /// Integer literal
    Int(Int<C>),
}

impl<'a, C> Display for Expr<'a, C> {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Expr::Id  (ref i) => Display::fmt(i, f),
            Expr::Int (ref n) => Display::fmt(n, f),
        }
    }
}

/*
impl<'a, C> AsmDisplay for Expr<'a, C> {
    #[inline]
    fn fmt(&self, f: &mut Formatter, s: &AsmSyntax) -> fmt::Result {
        // ?
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn fmt_id() {
        let e = Expr::Id(Id::new("a"));
        let s = format!("{}", &e);
        assert_eq!(s, "a");
    }

    #[test]
    pub fn fmt_int() {
        let e = Expr::Int(Int::from(42));
        let s = format!("{}", &e);
        assert_eq!(s, "0x2A");
    }
}

