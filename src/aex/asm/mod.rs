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

use std::fmt::{self, Debug, Display, Formatter};
use aex::ast::*;

pub mod att;
pub mod intel;
pub mod mit;

pub use self::att::*;
pub use self::intel::*;
pub use self::mit::*;

// -----------------------------------------------------------------------------

/// A type that is formattable as assembly code.
pub trait AsmDisplay<C = ()> {
    /// Formats the value as assembly code, using the given formatter and
    /// assembly style.
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle<C>) -> fmt::Result;
}

impl<'a, T, C> Display for Asm<'a, T, C> where T: AsmDisplay<C> + ?Sized {
    /// Formats the value using the given formatter.
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Asm(value, style) = *self;
        value.fmt(f, style)
    }
}

// -----------------------------------------------------------------------------

/// An assembly-formattable value paired with an assembly style.
#[derive(Clone, Copy, Debug)]
pub struct Asm<'a, T: 'a + AsmDisplay<C> + ?Sized, C: 'a>(
    /// Assembly-formattable value.
    pub &'a T,
    /// Assembly code style.
    pub &'a AsmStyle<C>
);

// -----------------------------------------------------------------------------

/// An assembly code style.
pub trait AsmStyle<C> : Debug {
    /// Writes an identifier to the given formatter in this assembly style.
    fn write_id(&self, f: &mut Formatter, id: &Id<C>) -> fmt::Result {
        f.write_str(id.name)
    }

    fn write_int(&self, f: &mut Formatter, num: &Int<C>) -> fmt::Result {
        write!(f, "{}", num.value)
    }
/*
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
*/
}

// -----------------------------------------------------------------------------

/// Asserts that formatting the given value in the given assembly style yields
/// the expected string.
#[cfg(test)]
pub fn assert_display<T: AsmDisplay<C>, C>(v: &T, s: &AsmStyle<C>, asm: &str) {
    assert_eq!(format!("{0}", Asm(v, s)), asm);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DefaultStyle;
    impl<C> AsmStyle<C> for DefaultStyle { }

    #[test]
    fn write_id() {
        assert_display(&Id::new("a"), &DefaultStyle, "a")
    }

    #[test]
    fn write_num() {
        assert_display(&Int::from(42), &DefaultStyle, "42")
    }
}

