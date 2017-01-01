// ColdFire Index Registers
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

use aex::asm::{AsmDisplay, AsmStyle};
use super::{AddrReg, DataReg};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Index {
    Data(DataReg),
    Addr(AddrReg),
}

impl Index {
    pub fn decode(word: u16, pos: u8) -> Self {
        let reg = (word >> pos     & 0b111) as u8;
        let da  = (word >> pos + 3 & 0b__1) as u8;

        match da {
            0 => Index::Data(DataReg::with_num(reg)),
            _ => Index::Addr(AddrReg::with_num(reg)),
        }
    }
}

impl AsmDisplay for Index {
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle) -> fmt::Result {
        match *self {
            Index::Data(ref r) => r.fmt(f, s),
            Index::Addr(ref r) => r.fmt(f, s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{A3, D5};

    #[test]
    fn decode_data() {
        let index = Index::decode(0b_0_101_00000, 5);
        assert_eq!(index, Index::Data(D5));
    }

    #[test]
    fn decode_addr() {
        let index = Index::decode(0b_1_011_00000, 5);
        assert_eq!(index, Index::Addr(A3));
    }
}

