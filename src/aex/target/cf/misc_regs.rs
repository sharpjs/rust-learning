// ColdFire Miscellaneous Registers
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

use std::fmt::{self, Formatter};

use aex::fmt::{AsmDisplay, AsmStyle};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PcReg;

impl AsmDisplay for PcReg {
    #[inline]
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle) -> fmt::Result {
        s.write_reg(f, "pc")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aex::fmt::*;

    #[test]
    fn display() {
        assert_eq!( format!("{0}", Asm(&PcReg, &GAS_STYLE)), "%pc" );
    }
}

