// Unary Operators
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
    #[inline]
    fn precedence(&self) -> Prec {
        self.prec()
    }
}

