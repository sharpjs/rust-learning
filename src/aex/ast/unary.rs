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
use aex::fmt::{Code, Style};
use aex::ast::{Assoc, Expr, Node, Op, Prec, Precedence};

/// A unary operator expression.
#[derive(Clone, Debug)]
pub struct Unary<'a, A = ()> {
    /// Operator.
    pub op: UnaryOp,

    /// Subexpression.
    pub expr: Box<Expr<'a, A>>,

    /// Annotation.
    pub ann: A,
}

impl<'a> Unary<'a> {
    /// Creates a new `Unary` with the given subexpression and with `()`
    /// annotation.
    pub fn new<E>(op: UnaryOp, expr: E) -> Self
    where E: Into<Box<Expr<'a>>> {
        Self::new_with_ann(op, expr, ())
    }
}

impl<'a, A> Unary<'a, A> {
    /// Creates a new `Unary` with the given subexpression and annotation.
    pub fn new_with_ann<E>(op: UnaryOp, expr: E, ann: A) -> Self
    where E: Into<Box<Expr<'a, A>>> {
        Unary { op: op, expr: expr.into(), ann: ann }
    }
}

impl<'a, A> Node for Unary<'a, A> {
    /// Annotation type.
    type Ann = A;

    /// Gets the annotation for this node.
    fn ann(&self) -> &A { &self.ann }
}

impl<'a, A> Precedence for Unary<'a, A> {
    /// Gets the operator precedence level.
    /// Higher values mean higher precendence.
    #[inline]
    fn precedence(&self) -> Prec {
        self.op.precedence()
    }
}

impl<'a, A> Display for Unary<'a, A> {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::Fixity::*;

        match self.op.fixity() {
            Prefix  => write!(f, "{}{}", self.op, self.expr),
            Postfix => write!(f, "{}{}", self.expr, self.op),
        }
    }
}

impl<'a, A> Code for Unary<'a, A> {
    /// Formats the value as code, using the given formatter and style.
    fn fmt<S: Style<A> + ?Sized>
          (&self, f: &mut Formatter, s: &S, p: Prec) -> fmt::Result {
        s.write_unary(f, self, p)
    }
}

/// Unary operators
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
    // Test (comparison with 0)
    Tst,
}

impl Op for UnaryOp {
    /// Gets the operator precedence level.
    fn prec(&self) -> Prec {
        self.precedence()
    }

    /// Gets the operator associativity.
    fn assoc(&self) -> Assoc {
        use self::UnaryOp::*;
        use super::Assoc::*;

        match *self {
            PostInc => Left,
            PostDec => Left,
            PreInc  => Right,
            PreDec  => Right,
            Ref     => Right,
            Clr     => Right,
            Not     => Right,
            Neg     => Right,
            Tst     => Left,
        }
    }
}

/// Unary operator fixity (prefix or postfix)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fixity { Prefix, Postfix }

impl UnaryOp {
    /// Gets the fixity (prefix or postfix) of this operator.
    #[inline]
    pub fn fixity(&self) -> Fixity {
        match self.precedence() {
            Prec::Prefix => Fixity::Prefix,
            _            => Fixity::Postfix,
        }
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
            Tst     => "?",
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
            Tst     => Comparison,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Id;
    use aex::fmt::{Styled, IntelStyle};

    #[test]
    fn new() {
        let e = pre_dec();
        assert_pre_dec(&e);
    }

    #[test]
    fn new_with_ann() {
        let e = pre_dec_with_ann();
        assert_pre_dec(&e);
        assert_eq!(e.ann, 42);
    }

    #[test]
    fn precedence() {
        let e = pre_dec();
        assert_eq!(e.precedence(), Prec::Prefix);
    }

    #[test]
    fn fixity() {
        let pre  = pre_dec();
        let post = post_inc();
        let test = Unary::new(UnaryOp::Tst, Expr::Id(Id::new("a")));
        assert_eq!(pre .op.fixity(), Fixity::Prefix);
        assert_eq!(post.op.fixity(), Fixity::Postfix);
        assert_eq!(test.op.fixity(), Fixity::Postfix);
    }

    #[test]
    fn fmt() {
        let pre  = pre_dec() .to_string();
        let post = post_inc().to_string();
        assert_eq!(pre,  "--a");
        assert_eq!(post, "a++");
    }

    #[test]
    fn fmt_asm() {
        let pre  = Styled::new(&pre_dec(),  &IntelStyle).to_string();
        let post = Styled::new(&post_inc(), &IntelStyle).to_string();
        assert_eq!(pre,  "--a");
        assert_eq!(post, "a++");
    }

    fn pre_dec<'a>() -> Unary<'a> {
        Unary::new(
            UnaryOp::PreDec,
            Expr::Id(Id::new("a"))
        )
    }

    fn post_inc<'a>() -> Unary<'a> {
        Unary::new(
            UnaryOp::PostInc,
            Expr::Id(Id::new("a"))
        )
    }

    fn pre_dec_with_ann<'a>() -> Unary<'a, usize> {
        Unary::new_with_ann(
            UnaryOp::PreDec,
            Expr::Id(Id::new_with_ann("a", 123)),
            42
        )
    }

    fn assert_pre_dec<A>(e: &Unary<A>) {
        assert_eq!(e.op, UnaryOp::PreDec);

        match *e.expr {
            Expr::Id(ref i) => assert_eq!(i.name, "a"),
            _ => panic!("Subexpression not Id")
        }
    }
}

