// ColdFire Program Counter + Displacement + Index Mode
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
use std::io::{self, Read};
use byteorder::{BigEndian as BE, ReadBytesExt};

use aex::fmt::{Code, Style};
use aex::ast::Expr;
use super::{Index, Scale};

use super::PcReg;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PcDispIdx<'a> {
    pub disp:  Expr<'a>,
    pub index: Index,
    pub scale: Scale,
}

impl<'a> PcDispIdx<'a> {
    pub fn decode<R: Read>(more: &mut R) -> io::Result<Self> {
        let ext = more.read_u16::<BE>()?;

        Ok(PcDispIdx {
            disp:  Expr::Int(ext as u8 as u32),
            index: Index::decode(ext, 12),
            scale: Scale::decode(ext, 9)?,
        })
    }
}

impl<'a> Code for PcDispIdx<'a> {
    fn fmt(&self, f: &mut Formatter, s: &Style) -> fmt::Result {
        s.write_base_disp_idx(f, &PcReg, &self.disp, &self.index, &self.scale)
    }
}

#[cfg(test)]
mod tests {
    use aex::fmt::*;
    use aex::ast::Expr;
    use super::*;
    use super::super::{D3, Index, Scale};

    #[test]
    fn display() {
        let x = PcDispIdx {
            disp:  Expr::Int(42),
            index: Index::Data(D3),
            scale: Scale::Word,
        };
        assert_display(&x, &GAS_STYLE, "42(%pc, %d3*2)");
    }
}

