// ColdFire Program Counter + Displacement Mode
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

use super::PcReg;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PcDisp<'a> {
    pub disp: Expr<'a>
}

impl<'a> PcDisp<'a> {
    pub fn decode<R: Read>(more: &mut R) -> io::Result<Self> {
        let ext = more.read_u16::<BE>()?;

        Ok(PcDisp { disp: Expr::Int(ext as u32) })
    }
}

impl<'a> Code for PcDisp<'a> {
    fn fmt(&self, f: &mut Formatter, s: &Style) -> fmt::Result {
        s.write_base_disp(f, &PcReg, &self.disp)
    }
}

#[cfg(test)]
mod tests {
    use aex::fmt::*;
    use aex::ast::Expr;
    use super::*;

    #[test]
    fn display() {
        let x = PcDisp { disp: Expr::Int(42) };
        assert_display(&x, &GAS_STYLE, "42(%pc)");
    }
}

