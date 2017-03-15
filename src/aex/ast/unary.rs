// AST: Unary Expressions
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
use aex::asm::{Asm, AsmDisplay, AsmStyle};
use aex::ast::{Expr, Prec, Precedence};

/// A binary operator expression.
#[derive(Clone, Debug)]
pub struct Unary<'a, C = ()> {
    /// Operator.
    pub op: UnaryOp,

    /// Subexpression.
    pub expr: Box<Expr<'a, C>>,

    /// Context value.
    pub context: C,
}

impl<'a> Unary<'a> {
    /// Creates a new `Unary` with the given subexpression and with `()`
    /// context.
    pub fn new<E>(op: UnaryOp, expr: E) -> Self
    where E: Into<Box<Expr<'a>>> {
        Unary { op: op, expr: expr.into(), context: () }
    }
}

impl<'a, C> Unary<'a, C> {
    /// Creates a new `Unary` with the given subexpression and context.
    pub fn new_with_context<E>(op: UnaryOp, expr: E, ctx: C) -> Self
    where E: Into<Box<Expr<'a, C>>> {
        Unary { op: op, expr: expr.into(), context: ctx }
    }
}

impl<'a, C> Precedence for Unary<'a, C> {
    /// Gets the operator precedence level.
    /// Higher values mean higher precendence.
    #[inline]
    fn precedence(&self) -> Prec {
        self.op.precedence()
    }
}

impl<'a, C> Display for Unary<'a, C> {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.op.is_postfix() {
            write!(f, "{}{}", self.expr, self.op)
        } else {
            write!(f, "{}{}", self.op, self.expr)
        }
    }
}

impl<'a, C> AsmDisplay<C> for Unary<'a, C> {
    /// Formats the value as assembly code, using the given formatter and
    /// assembly style.
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle<C>) -> fmt::Result {
        if self.op.is_postfix() {
            write!(f, "{}{}", Asm(&*self.expr, s), self.op)
        } else {
            write!(f, "{}{}", self.op, Asm(&*self.expr, s))
        }
    }
}

/// A binary operator expression.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnaryOp {
    // Post-increment
    PostInc,
    // Post-decrement
    PostDec,
    // Pre-increment
    PreInc,
    // Pre-decrement
    PreDec,
    // Reference (address-of)
    Ref,
    // Clear
    Clr,
    // Bitwise Not (1's complement)
    Not,
    // Negate (2's complement)
    Neg,
}

impl UnaryOp {
    /// Tests if a `UnaryOp` is a postfix operator (instead of prefix).
    #[inline]
    pub fn is_postfix(&self) -> bool {
        self.precedence() == Prec::Postfix
    }
}

impl Display for UnaryOp {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::UnaryOp::*;

        f.write_str(match *self {
            PostInc => "++",
            PostDec => "--",
            PreInc  => "++",
            PreDec  => "--",
            Ref     => "&",
            Clr     => "!",
            Not     => "~",
            Neg     => "-",
        })
    }
}

impl Precedence for UnaryOp {
    fn precedence(&self) -> Prec {
        use self::UnaryOp::*;
        use super::Prec::*;

        match *self {
            PostInc => Postfix,
            PostDec => Postfix,
            PreInc  => Prefix,
            PreDec  => Prefix,
            Ref     => Prefix,
            Clr     => Prefix,
            Not     => Prefix,
            Neg     => Prefix,
        }
    }
}

/*
impl<C> AsmDisplay<C> for UnaryOp {
    /// Formats the value as assembly code, using the given formatter and
    /// assembly style.
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle<C>) -> fmt::Result {
        // ...
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Id;
    use aex::asm::{Asm, IntelStyle};

    #[test]
    fn new() {
        let e = unary();
        assert_neg_a(&e);
    }

    #[test]
    fn new_with_context() {
        let e = unary_with_context();
        assert_neg_a(&e);
        assert_eq!(e.context, 42);
    }

    #[test]
    fn precedence() {
        let e = unary();
        let p = e.precedence();
        assert_eq!(p, Prec::Prefix);
    }

    #[test]
    fn fmt() {
        let e = unary();
        let s = e.to_string();
        assert_eq!(s, "-a");
    }

    #[test]
    fn fmt_asm() {
        let e = unary();
        let s = Asm(&e, &IntelStyle).to_string();
        assert_eq!(s, "-a");
    }

    fn unary<'a>() -> Unary<'a> {
        Unary::new(
            UnaryOp::Neg,
            Expr::Id(Id::new("a"))
        )
    }

    fn unary_with_context<'a>() -> Unary<'a, usize> {
        Unary::new_with_context(
            UnaryOp::Neg,
            Expr::Id(Id::new_with_context("a", 123)),
            42
        )
    }

    fn assert_neg_a<C>(e: &Unary<C>) {
        assert_eq!(e.op, UnaryOp::Neg);

        match *e.expr {
            Expr::Id(ref i) => assert_eq!(i.name, "a"),
            _ => panic!("Subexpression not Id")
        }
    }
}

