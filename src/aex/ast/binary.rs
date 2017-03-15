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
use aex::asm::{Asm, AsmDisplay, AsmStyle};
use aex::ast::{Expr, Precedence};

/// A binary operator expression.
#[derive(Clone, Debug)]
pub struct Binary<'a, C = ()> {
    /// Operator.
    pub op: BinaryOp,

    /// Left subexpression.
    pub lhs: Box<Expr<'a, C>>,

    /// Right subexpression.
    pub rhs: Box<Expr<'a, C>>,

    /// Context value.
    pub context: C,
}

impl<'a> Binary<'a> {
    /// Creates a new `Binary` with the given subexpressions and with `()`
    /// context.
    pub fn new<L, R>(op: BinaryOp, lhs: L, rhs: R) -> Self
    where L: Into<Box<Expr<'a>>>,
          R: Into<Box<Expr<'a>>> {
        Binary { op: op, lhs: lhs.into(), rhs: rhs.into(), context: () }
    }
}

impl<'a, C> Binary<'a, C> {
    /// Creates a new `Binary` with the given subexpressions and context.
    pub fn new_with_context<L, R>(op: BinaryOp, lhs: L, rhs: R, ctx: C) -> Self
    where L: Into<Box<Expr<'a, C>>>,
          R: Into<Box<Expr<'a, C>>> {
        Binary { op: op, lhs: lhs.into(), rhs: rhs.into(), context: ctx }
    }
}

impl<'a, C> Precedence for Binary<'a, C> {
    /// Gets the operator precedence level.
    /// Higher values mean higher precendence.
    #[inline]
    fn precedence(&self) -> usize {
        self.op.precedence()
    }
}

impl<'a, C> Display for Binary<'a, C> {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({} {} {})", self.lhs, self.op, self.rhs)
    }
}

impl<'a, C> AsmDisplay<C> for Binary<'a, C> {
    /// Formats the value as assembly code, using the given formatter and
    /// assembly style.
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle<C>) -> fmt::Result {
        write!(f, "({} {} {})", Asm(&*self.lhs, s), self.op, Asm(&*self.rhs, s))
    }
}

/// A binary operator expression.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinaryOp {
    /// Multiply
    Mul,
    /// Divide
    Div,
    /// Modulo
    Mod,
    /// Add
    Add,
    /// Subtract
    Sub,
    /// Shift left
    Shl,
    /// Shift right
    Shr,
    /// Rotate left
    Rol,
    /// Rotate right
    Ror,
    /// Rotate left through carry
    Rcl,
    /// Rotate right through carry
    Rcr,
    /// Bitwise AND
    And,
    /// Bitwise XOR
    Xor,
    /// Bitwise OR
    Or,
    /// Compare
    Cmp,
    /// Equal to
    Eq,
    /// Not equal to
    Ne,
    /// Less than
    Lt,
    /// Less than or equal to
    Le,
    /// Greater than
    Gt,
    /// Greater than or equal to
    Ge,
    /// Assign
    Mov,
}

impl Display for BinaryOp {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::BinaryOp::*;

        f.write_str(match *self {
            Mul => "*",
            Div => "/",
            Mod => "%",
            Add => "+",
            Sub => "-",
            Shl => "<<",
            Shr => ">>",
            Rol => "<<|",
            Ror => "|>>",
            Rcl => "<<%",
            Rcr => "%>>",
            And => "&",
            Xor => "^",
            Or  => "|",
            Cmp => "<>",
            Eq  => "==",
            Ne  => "!=",
            Lt  => "<",
            Le  => "<=",
            Gt  => ">",
            Ge  => ">=",
            Mov => "=",
        })
    }
}

impl Precedence for BinaryOp {
    fn precedence(&self) -> usize {
        use self::BinaryOp::*;

        match *self {
            Mul => 8,
            Div => 8,
            Mod => 8,
            Add => 7,
            Sub => 7,
            Shl => 6,
            Shr => 6,
            Rol => 6,
            Ror => 6,
            Rcl => 6,
            Rcr => 6,
            And => 5,
            Xor => 4,
            Or  => 3,
            Cmp => 2,
            Eq  => 1,
            Ne  => 1,
            Lt  => 1,
            Le  => 1,
            Gt  => 1,
            Ge  => 1,
            Mov => 0,
        }
    }
}

/*
impl<C> AsmDisplay<C> for BinaryOp {
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
        let b = binary();
        assert_add_a_b(&b);
    }

    #[test]
    fn new_with_context() {
        let b = binary_with_context();
        assert_add_a_b(&b);
        assert_eq!(b.context, 42);
    }

    #[test]
    fn precedence() {
        let b = binary();
        assert!(b.precedence() == BinaryOp::Add.precedence());
        assert!(b.precedence()  > 0);
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
        let s = Asm(&b, &IntelStyle).to_string();
        assert_eq!(s, "(a + b)");
    }

    fn binary<'a>() -> Binary<'a> {
        Binary::new(
            BinaryOp::Add,
            Expr::Id(Id::new("a")),
            Expr::Id(Id::new("b"))
        )
    }

    fn binary_with_context<'a>() -> Binary<'a, usize> {
        Binary::new_with_context(
            BinaryOp::Add,
            Expr::Id(Id::new_with_context("a", 123)),
            Expr::Id(Id::new_with_context("b", 456)),
            42
        )
    }

    fn assert_add_a_b<C>(b: &Binary<C>) {
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

