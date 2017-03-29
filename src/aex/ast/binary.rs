// AST: Binary Expressions
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
use aex::ast::{/*Assoc,*/ BinaryOp, Expr, Node, Prec, Precedence};

/// A binary operator expression.
#[derive(Clone, Debug)]
pub struct Binary<'a, A = ()> {
    /// Operator.
    pub op: BinaryOp,

    /// Left subexpression.
    pub lhs: Box<Expr<'a, A>>,

    /// Right subexpression.
    pub rhs: Box<Expr<'a, A>>,

    /// Annotation.
    pub ann: A,
}

impl<'a> Binary<'a> {
    /// Creates a new `Binary` with the given subexpressions and with `()`
    /// annotation.
    pub fn new<L, R>(op: BinaryOp, lhs: L, rhs: R) -> Self
    where L: Into<Box<Expr<'a>>>,
          R: Into<Box<Expr<'a>>> {
        Self::new_with_ann(op, lhs, rhs, ())
    }
}

impl<'a, A> Binary<'a, A> {
    /// Creates a new `Binary` with the given subexpressions and annotation.
    pub fn new_with_ann<L, R>(op: BinaryOp, lhs: L, rhs: R, ann: A) -> Self
    where L: Into<Box<Expr<'a, A>>>,
          R: Into<Box<Expr<'a, A>>> {
        Binary { op: op, lhs: lhs.into(), rhs: rhs.into(), ann: ann }
    }
}

impl<'a, A> Node for Binary<'a, A> {
    /// Annotation type.
    type Ann = A;

    /// Gets the annotation for this node.
    fn ann(&self) -> &A { &self.ann }
}

impl<'a, A> Precedence for Binary<'a, A> {
    /// Gets the operator precedence level.
    /// Higher values mean higher precendence.
    #[inline]
    fn precedence(&self) -> Prec {
        self.op.precedence()
    }
}

impl<'a, A> Display for Binary<'a, A> {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({} {} {})", self.lhs, self.op, self.rhs)
    }
}

impl<'a, A> Code for Binary<'a, A> {
    /// Formats the value as code, using the given formatter and style.
    fn fmt<S: Style<A> + ?Sized>
          (&self, f: &mut Formatter, s: &S) -> fmt::Result {
        s.write_binary(f, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Id;
    use aex::fmt::{ToStyled, IntelStyle};

    #[test]
    fn new() {
        let b = binary();
        assert_add_a_b(&b);
    }

    #[test]
    fn new_with_ann() {
        let b = binary_with_ann();
        assert_add_a_b(&b);
        assert_eq!(b.ann, 42);
    }

    #[test]
    fn precedence() {
        let b = binary();
        let p = b.precedence();
        assert_eq!(p, Prec::Additive);
    }

    #[test]
    fn fmt() {
        let b = binary();
        let s = b.to_string();
        assert_eq!(s, "(a + b)");
    }

    #[test]
    fn fmt_asm() {
        let b = binary();
        let s = b.styled(&IntelStyle).to_string();
        assert_eq!(s, "a + b");
    }

    fn binary<'a>() -> Binary<'a> {
        Binary::new(
            BinaryOp::Add,
            Expr::Id(Id::new("a")),
            Expr::Id(Id::new("b"))
        )
    }

    fn binary_with_ann<'a>() -> Binary<'a, usize> {
        Binary::new_with_ann(
            BinaryOp::Add,
            Expr::Id(Id::new_with_ann("a", 123)),
            Expr::Id(Id::new_with_ann("b", 456)),
            42
        )
    }

    fn assert_add_a_b<A>(b: &Binary<A>) {
        assert_eq!(b.op, BinaryOp::Add);

        match *b.lhs {
            Expr::Id(ref i) => assert_eq!(i.name, "a"),
            _ => panic!("LHS not Id")
        }

        match *b.rhs {
            Expr::Id(ref i) => assert_eq!(i.name, "b"),
            _ => panic!("LHS not Id")
        }
    }
}

