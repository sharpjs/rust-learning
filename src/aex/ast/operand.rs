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
pub enum Operand<'a, A = ()> {

    /// A constant expression.
    Constant(Expr<'a, A>),

    /// A register.
    Register(Id<'a, A>),

    /// A memory location.
    Indirect(Indirect<'a, A>),
}

/// A reference to a memory location.
#[derive(Clone, Debug)]
pub struct Indirect<'a, A = ()> {

    /// Offsets that sum to the effective address.
    pub offsets: Vec<Offset<'a, A>>,

    /// Annotation.
    pub ann: A,
}

/// An offset in an indirect operand.
#[derive(Clone, Debug)]
pub enum Offset<'a, A = ()> {

    /// Constant displacement.
    Disp(Disp<'a, A>),

    /// Base register.
    Base(Id<'a, A>),

    /// Register with pre-decrement.
    PreDec(Id<'a, A>),

    /// Register with post-increment.
    PostInc(Id<'a, A>),

    /// Register scaled by logical left shift.
    Index(Index<'a, A>),
}

/// A constant displacement in an indirect operand.
#[derive(Clone, Debug)]
pub struct Disp<'a, A = ()> {

    /// The amount of displacement.
    pub value: Expr<'a, A>,

    /// The kind of displacement: near, far, etc.
    pub kind: DispKind,

    /// Annotation.
    pub ann: A,
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
pub struct Index<'a, A = ()> {

    /// The index register.
    pub reg: Id<'a, A>,

    /// The scaling applied to the index register.
    pub scale: Option<Scale<'a, A>>,

    /// Annotation.
    pub ann: A,
}

/// A scaling operation in an indirect operand.
#[derive(Clone, Debug)]
pub enum Scale<'a, A = ()> {

    /// Logical left shift.
    Lsl(Operand<'a, A>),

    /// Logical right shift. (ARM)
    Lsr(Operand<'a, A>),

    /// Arithmetic right shift. (ARM)
    Asr(Operand<'a, A>),

    /// Rotate right. (ARM)
    Ror(Operand<'a, A>),

    /// Rotate right through carry. (ARM)
    Rrc(Operand<'a, A>),
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    #[test]
//    fn fmt() {
//    }
//}

