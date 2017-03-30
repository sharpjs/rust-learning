// Operators
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

mod unary;
mod binary;

pub use self::unary::*;
pub use self::binary::*;

// -----------------------------------------------------------------------------

/// Trait for operator types.
pub trait Op: HasPrec + HasAssoc {}

// -----------------------------------------------------------------------------

/// Trait for types that expose operator precedence level.
pub trait HasPrec {
    /// Gets the operator precedence level.
    fn prec(&self) -> Prec;
}

/// Operator precedence levels.
///
/// Higher values represent higher precedence.
///
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Prec {
    Statement,          // Lowest precedence
    Assignment,
    Conditional,
    Comparison,
    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,
    BitwiseShift,
    Additive,
    Multiplicative,
    Casting,
    Prefix,
    Postfix,
    Atomic,             // Highest precedence
}

/// Minimum operator precedence level.
pub const PREC_MIN: Prec = Prec::Statement;

/// Maximum operator precedence level.
pub const PREC_MAX: Prec = Prec::Atomic;

impl Prec {
    /// Returns whether an expression at this precedence level should appear
    /// grouped (parenthesized) when inside the given outer precedence level.
    ///
    /// If this precedence level and `outer` are equal, returns `if_eq`.
    ///
    pub fn should_group(self, outer: Prec, if_eq: bool) -> bool {
        use std::cmp::Ordering::*;

        match self.cmp(&outer) {
            Less    => true,
            Equal   => if_eq,
            Greater => false,
        }
    }
}

// -----------------------------------------------------------------------------

/// Trait for types that expose operator associativity.
pub trait HasAssoc {
    /// Gets the operator associativity.
    fn assoc(&self) -> Assoc;
}

/// Operator associativity.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Assoc {
    /// Left-associative
    Left,
    /// Right-associative
    Right,
    /// Non-associative
    Non,
}

