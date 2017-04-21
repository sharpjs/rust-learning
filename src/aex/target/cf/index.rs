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

use aex::ast::Reg;
use aex::fmt::ToCode;

use super::{AddrReg, DataReg};
use self::Index::*;

/// ColdFire index registers.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Index {
    /// Data register used as an index register.
    Data(DataReg),
    /// Address register used as an index register.
    Addr(AddrReg),
}

impl Index {
    /// Decodes an index register from the given instruction bits.
    pub fn decode(word: u16, pos: u8) -> Self {
        let reg = (word >> pos     & 0b111) as u8;
        let da  = (word >> pos + 3 & 0b__1) as u8;

        match da {
            0 => Data(DataReg::with_num(reg)),
            _ => Addr(AddrReg::with_num(reg)),
        }
    }
}

impl<A> ToCode<A> for Index {
    type Output = Reg<'static, A>;

    /// Converts to a code-formattable value with the given annotation.
    #[inline]
    fn to_code(&self, ann: A) -> Self::Output {
        match *self {
            Data(ref r) => r.to_code(ann),
            Addr(ref r) => r.to_code(ann),
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
        assert_eq!(index, Data(D5));
    }

    #[test]
    fn decode_addr() {
        let index = Index::decode(0b_1_011_00000, 5);
        assert_eq!(index, Addr(A3));
    }

    #[test]
    fn to_code() {
        let c = Addr(A3).to_code(42);

        assert_eq!(c.name, "a3");
        assert_eq!(c.ann,   42 );
    }
}

