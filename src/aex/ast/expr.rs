// Expressions
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
pub enum Expr<'a, A = ()> {
    /// Identifier
    Id(Id<'a, A>),

    /// Integer literal
    Int(Int<A>),

    /// Register
    Reg(Reg<'a, A>),

    /// Unary expression
    Unary(Unary<'a, A>),

    /// Binary expression
    Binary(Binary<'a, A>),
}

impl<'a, A> Node for Expr<'a, A> {
    /// Annotation type.
    type Ann = A;

    /// Gets the annotation for this node.
    fn ann(&self) -> &A {
        match *self {
            Expr::Id     (ref i) => i.ann(),
            Expr::Int    (ref i) => i.ann(),
            Expr::Reg    (ref r) => r.ann(),
            Expr::Unary  (ref u) => u.ann(),
            Expr::Binary (ref b) => b.ann(),
        }
    }
}

impl<'a, A> HasPrec for Expr<'a, A> {
    /// Gets the operator precedence level.
    fn prec(&self) -> Prec {
        match *self {
            Expr::Id     (ref i) => i.prec(),
            Expr::Int    (ref i) => i.prec(),
            Expr::Reg    (ref r) => r.prec(),
            Expr::Unary  (ref u) => u.prec(),
            Expr::Binary (ref b) => b.prec(),
        }
    }
}

impl<'a, A> Display for Expr<'a, A> {
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

impl<'a, A> Code for Expr<'a, A> {
    /// Formats the value as code, using the given formatter and style.
    fn fmt<S: Style<A> + ?Sized>
          (&self, f: &mut Formatter, s: &S) -> fmt::Result {
        match *self {
            Expr::Id     (ref i) => Code::fmt(i, f, s),
            Expr::Int    (ref i) => Code::fmt(i, f, s),
            Expr::Reg    (ref r) => Code::fmt(r, f, s),
            Expr::Unary  (ref u) => Code::fmt(u, f, s),
            Expr::Binary (ref b) => Code::fmt(b, f, s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prec() {
        let e = Binary::new(
            BinaryOp::Add,
            Expr::Id(Id::new("a")),
            Expr::Id(Id::new("b"))
        );
        assert_eq!(e.prec(), Prec::Additive);
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

