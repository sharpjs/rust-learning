
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

/// Types that have operator metadata.
pub trait Op {
    /// Gets the operator precedence level.
    fn prec(&self) -> Prec;

    /// Gets the operator associativity.
    fn assoc(&self) -> Assoc;
}

// -----------------------------------------------------------------------------

/// Operator associativity.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Assoc {
    /// Left-associative
    Left,

    /// Right-associative
    Right,

    /// Non-associative
    None,
}

// -----------------------------------------------------------------------------

/// Operator precedence levels.
///
/// Higher values indicate higher precedence.
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

// -----------------------------------------------------------------------------

// TODO: Can this go away in favor of Op?

/// Trait for types that have operator precedence.
pub trait Precedence {
    /// Gets the operator precedence level.
    fn precedence(&self) -> Prec;
}

