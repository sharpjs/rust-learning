// Binary Operators
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
use super::{Assoc, Op, Prec, Precedence};

/// Binary operators
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

impl Op for BinaryOp {
    /// Gets the operator precedence level.
    fn prec(&self) -> Prec {
        use self::BinaryOp::*;
        use super::Prec::*;

        match *self {
            Mul => Multiplicative,
            Div => Multiplicative,
            Mod => Multiplicative,
            Add => Additive,
            Sub => Additive,
            Shl => BitwiseShift,
            Shr => BitwiseShift,
            Rol => BitwiseShift,
            Ror => BitwiseShift,
            Rcl => BitwiseShift,
            Rcr => BitwiseShift,
            And => BitwiseAnd,
            Xor => BitwiseXor,
            Or  => BitwiseOr,
            Cmp => Comparison,
            Eq  => Conditional,
            Ne  => Conditional,
            Lt  => Conditional,
            Le  => Conditional,
            Gt  => Conditional,
            Ge  => Conditional,
            Mov => Assignment,
        }
    }

    /// Gets the operator associativity.
    fn assoc(&self) -> Assoc {
        use self::BinaryOp::*;
        use super::Assoc::*;

        match *self {
            Mul => Left,
            Div => Left,
            Mod => Left,
            Add => Left,
            Sub => Left,
            Shl => Left,
            Shr => Left,
            Rol => Left,
            Ror => Left,
            Rcl => Left,
            Rcr => Left,
            And => Left,
            Xor => Left,
            Or  => Left,
            Cmp => Left,
            Eq  => Non,
            Ne  => Non,
            Lt  => Non,
            Le  => Non,
            Gt  => Non,
            Ge  => Non,
            Mov => Right,
        }
    }
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
    fn precedence(&self) -> Prec {
        #[inline]
        self.prec()
    }
}

