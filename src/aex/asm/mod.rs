// Assembly Syntax
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

//use std::fmt::{self, Debug, Display, Formatter};

//pub use self::att::*;
//pub use self::intel::*;
//pub use self::mit::*;

//pub mod att;
//pub mod intel;
//pub mod mit;

use aex::ast::{Expr, Id};

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

/*
// -----------------------------------------------------------------------------

pub trait AsmDisplay {
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle) -> fmt::Result;
}

impl<'a, T> Display for Asm<'a, T> where T: AsmDisplay + ?Sized {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Asm(value, style) = *self;
        value.fmt(f, style)
    }
}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Asm<'a, T: 'a + ?Sized>(
    pub &'a T,
    pub &'a AsmStyle
);

// -----------------------------------------------------------------------------

pub trait AsmStyle : Debug {
    fn write_id<'a>(&self, f: &mut Formatter, value: &Id<'a>) -> fmt::Result {
        Ok(())
    }

    fn write_int(&self, f: &mut Formatter, value: u64) -> fmt::Result {
        write!(f, "{}", value)
    }

    fn write_reg(&self, f: &mut Formatter, name: &str) -> fmt::Result {
        f.write_str(name)
    }

    fn write_ind
        (&self, f: &mut Formatter, reg: &AsmDisplay)
        -> fmt::Result { Err(fmt::Error) }

    fn write_ind_predec
        (&self, f: &mut Formatter, reg: &AsmDisplay)
        -> fmt::Result { Err(fmt::Error) }

    fn write_ind_postinc
        (&self, f: &mut Formatter, reg: &AsmDisplay)
        -> fmt::Result { Err(fmt::Error) }

    fn write_base_disp
        (&self, f: &mut Formatter, base: &AsmDisplay, disp: &AsmDisplay)
        -> fmt::Result { Err(fmt::Error) }

    fn write_base_disp_idx
        (&self, f: &mut Formatter, base: &AsmDisplay, disp: &AsmDisplay, index: &AsmDisplay, scale: &AsmDisplay)
        -> fmt::Result { Err(fmt::Error) }

    fn write_scale
        (&self, f: &mut Formatter, scale: u8)
        -> fmt::Result { Err(fmt::Error) }
}

#[cfg(test)]
pub static GAS_STYLE: AttStyle = AttStyle {
    arg_spaces: true,
    reg_prefix: "%",
    imm_prefix: "#",
};

#[cfg(test)]
pub fn assert_display<T: AsmDisplay>(v: &T, s: &AsmStyle, asm: &str) {
    assert_eq!(format!("{0}", Asm(v, s)), asm);
}

// -----------------------------------------------------------------------------
*/

