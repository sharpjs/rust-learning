// Assembly Style
//
// This file is part of AEx.
// Copyright (C) 2016 Jeffrey Sharp
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

pub trait AsmDisplay {
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle) -> fmt::Result;
}

impl<'a, T> Display for Asm<'a, T> where T: AsmDisplay {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Asm(ref value, style) = *self;
        value.fmt(f, style)
    }
}

pub struct Asm<'a, T> (pub T, pub &'a AsmStyle);

#[derive(Clone, Debug)]
pub struct AsmStyle {
    pub arg_spaces:     bool,
    pub reg_prefix:     &'static str,
    pub imm_prefix:     &'static str,

    pub ind_style:      IndStyle,
    pub ind_open:       &'static str,
    pub ind_close:      &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum IndStyle {
    Math,   // [base + index*scale + disp]
    Comma,  // (disp, base, index.size*scale)
    Moto,   // disp(base, index.size*scale)
    Mit,    // base@(disp, index:size:scale)
}

pub static MY_STYLE: AsmStyle = AsmStyle {
    arg_spaces:     true,
    reg_prefix:     "",
    imm_prefix:     "",
    ind_style:      IndStyle::Math,
    ind_open:       "[",
    ind_close:      "]",
};

pub static GAS_STYLE: AsmStyle = AsmStyle {
    arg_spaces:     true,
    reg_prefix:     "%",
    imm_prefix:     "#",
    ind_style:      IndStyle::Moto,
    ind_open:       "(",
    ind_close:      ")",
};

impl AsmStyle {
    pub fn write_reg(&self, f: &mut Formatter, r: &str)
                    -> fmt::Result {
        f.write_str(self.reg_prefix)?;
        f.write_str(r)
    }

    pub fn write_base_disp<B: Display, D: Display>
                          (&self, f: &mut Formatter, b: &B, d: &D)
                          -> fmt::Result {
        match self.ind_style {
            IndStyle::Math => write!(
                f, "{open}{base}{sp}+{sp}{disp}{close}",
                open  = self.ind_open,
                close = self.ind_close,
                sp    = if self.arg_spaces {" "} else {""},
                base  = b,
                disp  = d
            ),
            IndStyle::Comma => write!(
                f, "{open}{base},{sp}{disp}{close}",
                open  = self.ind_open,
                close = self.ind_close,
                sp    = if self.arg_spaces {" "} else {""},
                base  = b,
                disp  = d
            ),
            IndStyle::Moto => write!(
                f, "{disp}{open}{base}{close}",
                open  = self.ind_open,
                close = self.ind_close,
                base  = b,
                disp  = d
            ),
            IndStyle::Mit => write!(
                f, "{base}@{open}{disp}{close}",
                open  = self.ind_open,
                close = self.ind_close,
                base  = b,
                disp  = d
            ),
        }
    }
}

#[cfg(test)]
pub fn assert_display<'a, T>(v: T, s: &'a AsmStyle, a: &str)
    where Asm<'a, T>: Display {

    assert_eq!(format!("{0}", Asm(v, s)), a);
}

