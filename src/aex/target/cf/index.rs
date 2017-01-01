// ColdFire Index Registers
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

use std::fmt::{self, Formatter};

use aex::asm::{AsmDisplay, AsmStyle};
use super::{AddrReg, DataReg};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum Scale {
    Byte = 1,
    Word = 2,
    Long = 4,
}

impl Scale {
    pub fn with_size(size: usize) -> Option<Self> {
        match size {
            1 => Some(Scale::Byte),
            2 => Some(Scale::Word),
            4 => Some(Scale::Long),
            _ => None,
        }
    }

    pub fn decode(word: u16, pos: u8) -> Option<Self> {
        let scale = word >> pos & 0b11;
        match scale {
            0 => Some(Scale::Byte),
            1 => Some(Scale::Word),
            2 => Some(Scale::Long),
            _ => None,
        }
    }
}

impl AsmDisplay for Scale {
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle) -> fmt::Result {
        s.write_scale(f, *self as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_size() {
        assert_eq!(Scale::with_size(0), None);
        assert_eq!(Scale::with_size(1), Some(Scale::Byte));
        assert_eq!(Scale::with_size(2), Some(Scale::Word));
        assert_eq!(Scale::with_size(4), Some(Scale::Long));
    }

    #[test]
    fn decode() {
        assert_eq!(Scale::decode(0b0000, 2), Some(Scale::Byte));
        assert_eq!(Scale::decode(0b0100, 2), Some(Scale::Word));
        assert_eq!(Scale::decode(0b1000, 2), Some(Scale::Long));
        assert_eq!(Scale::decode(0b1100, 2), None);
    }
}

