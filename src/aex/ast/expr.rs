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
use aex::fmt::{Code, Style};
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

    /// Unary expression
    Unary(Unary<'a, C>),

    /// Binary expression
    Binary(Binary<'a, C>),
}

impl<'a, C> Node for Expr<'a, C> {
    /// Type of the context value.
    type Context = C;

    /// Gets the context value.
    fn context(&self) -> &C {
        match *self {
            Expr::Id     (ref i) => i.context(),
            Expr::Int    (ref i) => i.context(),
            Expr::Reg    (ref r) => r.context(),
            Expr::Unary  (ref u) => u.context(),
            Expr::Binary (ref b) => b.context(),
        }
    }
}

impl<'a, C> Precedence for Expr<'a, C> {
    /// Gets the operator precedence level.
    fn precedence(&self) -> Prec {
        match *self {
            Expr::Id     (ref i) => i.precedence(),
            Expr::Int    (ref i) => i.precedence(),
            Expr::Reg    (ref r) => r.precedence(),
            Expr::Unary  (ref u) => u.precedence(),
            Expr::Binary (ref b) => b.precedence(),
        }
    }
}

impl<'a, C> Display for Expr<'a, C> {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Expr::Id     (ref i) => Display::fmt(i, f),
            Expr::Int    (ref i) => Display::fmt(i, f),
            Expr::Reg    (ref r) => Display::fmt(r, f),
            Expr::Unary  (ref u) => Display::fmt(u, f),
            Expr::Binary (ref b) => Display::fmt(b, f),
        }
    }
}

impl<'a, C> Code for Expr<'a, C> {
    /// Formats the value as assembly code, using the given formatter and
    /// assembly style.
    fn fmt<S: Style<C> + ?Sized>
          (&self, f: &mut Formatter, s: &S, p: Prec) -> fmt::Result {
        match *self {
            Expr::Id     (ref i) => Code::fmt(i, f, s, p),
            Expr::Int    (ref i) => Code::fmt(i, f, s, p),
            Expr::Reg    (ref r) => Code::fmt(r, f, s, p),
            Expr::Unary  (ref u) => Code::fmt(u, f, s, p),
            Expr::Binary (ref b) => Code::fmt(b, f, s, p),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precedence() {
        let e = Binary::new(
            BinaryOp::Add,
            Expr::Id(Id::new("a")),
            Expr::Id(Id::new("b"))
        );
        assert_eq!(e.precedence(), Prec::Additive);
    }

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

