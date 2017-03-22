// MIT Assembly Style
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

//use std::fmt::{self, Formatter};
use super::{/*Asm , AsmDisplay,*/ Style};

/// MIT assembly style.
#[derive(Clone, Debug)]
pub struct MitStyle {
    pub arg_spaces: bool,
    pub reg_prefix: &'static str,
    pub imm_prefix: &'static str,
}

impl Default for MitStyle {
    fn default() -> Self {
        MitStyle {
            arg_spaces: false,
            reg_prefix: "%",
            imm_prefix: "#",
        }
    }
}

impl<C> Style<C> for MitStyle {
/*
    fn write_reg(
        &self,
        f:    &mut Formatter,
        name: &str
    ) -> fmt::Result {
        write!(f, "{}{}", self.reg_prefix, name)
    }

    fn write_ind(
        &self,
        f:  &mut Formatter,
        ea: &AsmDisplay
    ) -> fmt::Result {
        write!(f, "{}@", Asm(ea, self))
    }

    fn write_ind_predec(
        &self,
        f:   &mut Formatter,
        reg: &AsmDisplay
    ) -> fmt::Result {
        write!(f, "{}@-", Asm(reg, self))
    }

    fn write_ind_postinc(
        &self,
        f:   &mut Formatter,
        reg: &AsmDisplay
    ) -> fmt::Result {
        write!(f, "{}@+", Asm(reg, self))
    }

    fn write_base_disp(
        &self,
        f:    &mut Formatter,
        base: &AsmDisplay,
        disp: &AsmDisplay
    ) -> fmt::Result {
        write!(
            f, "{base}@({disp})",
            base = Asm(base, self),
            disp = Asm(disp, self)
        )
    }

    fn write_base_disp_idx(
        &self,
        f:     &mut Formatter,
        base:  &AsmDisplay,
        disp:  &AsmDisplay,
        index: &AsmDisplay,
        scale: &AsmDisplay
    ) -> fmt::Result {
        write!(
            f, "{base}@({disp},{sp}{index}:{scale})",
            sp    = if self.arg_spaces {" "} else {""},
            base  = Asm(base,  self),
            disp  = Asm(disp,  self),
            index = Asm(index, self),
            scale = Asm(scale, self),
        )
    }

    fn write_scale(
        &self,
        f:     &mut Formatter,
        scale: u8
    ) -> fmt::Result {
        write!(f, "{}", scale)
    }
*/
}

