// AST: Operands
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

//use std::fmt::{self, Display, Formatter};
//use aex::fmt::{Code, Style};
use super::*;

/// An instruction operand.
#[derive(Clone, Debug)]
pub enum Operand<'a, C = ()> {

    /// A constant expression.
    Constant(Expr<'a, C>),

    /// A register.
    Register(Id<'a, C>),

    /// A memory location.
    Indirect(Indirect<'a, C>),
}

/// A reference to a memory location.
#[derive(Clone, Debug)]
pub struct Indirect<'a, C = ()> {

    /// Offsets that sum to the effective address.
    pub offsets: Vec<Offset<'a, C>>,

    /// A context value.
    pub ctx: C,
}

/// An offset in an indirect operand.
#[derive(Clone, Debug)]
pub enum Offset<'a, C = ()> {

    /// Constant displacement.
    Disp(Disp<'a, C>),

    /// Base register.
    Base(Id<'a, C>),

    /// Register with pre-decrement.
    PreDec(Id<'a, C>),

    /// Register with post-increment.
    PostInc(Id<'a, C>),

    /// Register scaled by logical left shift.
    Index(Index<'a, C>),
}

/// A constant displacement in an indirect operand.
#[derive(Clone, Debug)]
pub struct Disp<'a, C = ()> {

    /// The amount of displacement.
    pub value: Expr<'a, C>,

    /// The kind of displacement: near, far, etc.
    pub kind: DispKind,

    /// A context value.
    pub ctx: C,
}

/// Kinds of constant displacement in an indirect operand.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DispKind {

    /// Far displacement (ex: .L on M68K).
    Far,

    /// Near displacement (ex: .W on M68K).
    Near,
}

/// An scaled index register in an indirect operand.
#[derive(Clone, Debug)]
pub struct Index<'a, C = ()> {

    /// The index register.
    pub reg: Id<'a, C>,

    /// The scaling applied to the index register.
    pub scale: Option<Scale<'a, C>>,

    /// A context value.
    pub ctx: C,
}

/// A scaling operation in an indirect operand.
#[derive(Clone, Debug)]
pub enum Scale<'a, C = ()> {

    /// Logical left shift.
    Lsl(Operand<'a, C>),

    /// Logical right shift. (ARM)
    Lsr(Operand<'a, C>),

    /// Arithmetic right shift. (ARM)
    Asr(Operand<'a, C>),

    /// Rotate right. (ARM)
    Ror(Operand<'a, C>),

    /// Rotate right through carry. (ARM)
    Rrc(Operand<'a, C>),
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    #[test]
//    fn fmt() {
//    }
//}

