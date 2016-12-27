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
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Asm(value, style) = *self;
        value.fmt(f, style)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Asm<'a, T: 'a>(
    pub &'a T,
    pub &'a AsmStyle
);

#[derive(Clone, Debug)]
pub struct AsmStyle {
    pub arg_spaces:     bool,
    pub reg_prefix:     &'static str,
    pub imm_prefix:     &'static str,

    pub ind_style:      IndirectStyle,
    pub ind_open:       &'static str,
    pub ind_close:      &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum IndirectStyle {
    Intel,  // [base + index*scale + disp]
    Comma,  // (disp, base, index.size*scale)
    Moto,   // disp(base, index.size*scale)
    Mit,    // base@(disp, index:size:scale)
}

pub static MY_STYLE: AsmStyle = AsmStyle {
    arg_spaces:     true,
    reg_prefix:     "",
    imm_prefix:     "",
    ind_style:      IndirectStyle::Intel,
    ind_open:       "[",
    ind_close:      "]",
};

pub static GAS_STYLE: AsmStyle = AsmStyle {
    arg_spaces:     true,
    reg_prefix:     "%",
    imm_prefix:     "#",
    ind_style:      IndirectStyle::Moto,
    ind_open:       "(",
    ind_close:      ")",
};

impl AsmStyle {
    pub fn write_reg(&self, f: &mut Formatter, reg: &str) -> fmt::Result {
        f.write_str(self.reg_prefix)?;
        f.write_str(reg)
    }

    pub fn write_ind<R: AsmDisplay>
                    (&self, f: &mut Formatter, reg: &R)
                    -> fmt::Result {
        match self.ind_style {
            IndirectStyle::Intel => write!(f, "[{}]", Asm(reg, self)),
            IndirectStyle::Comma => write!(f, "({})", Asm(reg, self)),
            IndirectStyle::Moto  => write!(f, "({})", Asm(reg, self)),
            IndirectStyle::Mit   => write!(f, "{}@",  Asm(reg, self)),
        }
    }

    pub fn write_ind_postinc<R: AsmDisplay>
                            (&self, f: &mut Formatter, reg: &R)
                            -> fmt::Result {
        match self.ind_style {
            IndirectStyle::Intel => write!(f, "[{}++]", Asm(reg, self)),
            IndirectStyle::Comma => write!(f, "({})+",  Asm(reg, self)),
            IndirectStyle::Moto  => write!(f, "({})+",  Asm(reg, self)),
            IndirectStyle::Mit   => write!(f, "{}@+",   Asm(reg, self)),
        }
    }

    pub fn write_ind_predec<R: AsmDisplay>
                           (&self, f: &mut Formatter, reg: &R)
                           -> fmt::Result {
        match self.ind_style {
            IndirectStyle::Intel => write!(f, "[--{}]", Asm(reg, self)),
            IndirectStyle::Comma => write!(f, "-({})",  Asm(reg, self)),
            IndirectStyle::Moto  => write!(f, "-({})",  Asm(reg, self)),
            IndirectStyle::Mit   => write!(f, "{}@-",   Asm(reg, self)),
        }
    }

    pub fn write_base_disp<B: AsmDisplay, D: AsmDisplay>
                          (&self, f: &mut Formatter, base: &B, disp: &D)
                          -> fmt::Result {
        match self.ind_style {
            IndirectStyle::Intel => write!(
                f, "{open}{base}{sp}+{sp}{disp}{close}",
                open  = self.ind_open,
                close = self.ind_close,
                sp    = if self.arg_spaces {" "} else {""},
                base  = Asm(base, self),
                disp  = Asm(disp, self)
            ),
            IndirectStyle::Comma => write!(
                f, "{open}{base},{sp}{disp}{close}",
                open  = self.ind_open,
                close = self.ind_close,
                sp    = if self.arg_spaces {" "} else {""},
                base  = Asm(base, self),
                disp  = Asm(disp, self)
            ),
            IndirectStyle::Moto => write!(
                f, "{disp}{open}{base}{close}",
                open  = self.ind_open,
                close = self.ind_close,
                base  = Asm(base, self),
                disp  = Asm(disp, self)
            ),
            IndirectStyle::Mit => write!(
                f, "{base}@{open}{disp}{close}",
                open  = self.ind_open,
                close = self.ind_close,
                base  = Asm(base, self),
                disp  = Asm(disp, self)
            ),
        }
    }
}

#[cfg(test)]
pub fn assert_display<T: AsmDisplay>(v: &T, s: &AsmStyle, asm: &str) {
    assert_eq!(format!("{0}", Asm(v, s)), asm);
}

