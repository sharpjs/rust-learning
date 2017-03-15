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
use aex::asm::{AsmDisplay, AsmStyle};
use super::*;

/// An expression.
#[derive(Clone, Debug)]
pub enum Expr<'a, C = ()> {
    /// Identifier
    Id(Id<'a, C>),

    /// Integer literal
    Int(Int<C>),

    /// Register
    Reg(Reg<'a, C>),

    /// Binary operation
    Binary(Binary<'a, C>),
}

impl<'a, C> Precedence for Expr<'a, C> {
    /// Gets the operator precedence level.
    /// Higher values mean higher precendence.
    fn precedence(&self) -> usize {
        match *self {
            Expr::Id     (_)     => 12,
            Expr::Int    (_)     => 12,
            Expr::Reg    (_)     => 12,
            Expr::Binary (ref e) => e.precedence(),
        }
    }
}

impl<'a, C> Display for Expr<'a, C> {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Expr::Id     (ref i) => Display::fmt(i, f),
            Expr::Int    (ref n) => Display::fmt(n, f),
            Expr::Reg    (ref r) => Display::fmt(r, f),
            Expr::Binary (ref e) => Display::fmt(e, f),
        }
    }
}

impl<'a, C> AsmDisplay<C> for Expr<'a, C> {
    /// Formats the value as assembly code, using the given formatter and
    /// assembly style.
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle<C>) -> fmt::Result {
        match *self {
            Expr::Id     (ref i) => AsmDisplay::fmt(i, f, s),
            Expr::Int    (ref n) => AsmDisplay::fmt(n, f, s),
            Expr::Reg    (ref r) => AsmDisplay::fmt(r, f, s),
            Expr::Binary (ref e) => AsmDisplay::fmt(e, f, s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_id() {
        let e = Expr::Id(Id::new("a"));
        let s = format!("{}", e);
        assert_eq!(s, "a");
    }

    #[test]
    fn fmt_int() {
        let e = Expr::Int(Int::from(42));
        let s = format!("{}", e);
        assert_eq!(s, "0x2A");
    }

    #[test]
    fn fmt_reg() {
        let e = Expr::Reg(Reg::new("a"));
        let s = format!("{}", e);
        assert_eq!(s, "a");
    }
}

