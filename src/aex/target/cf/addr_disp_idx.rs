// ColdFire Address Register + Displacement + Index Mode
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
use aex::ast::Expr;
use super::{AddrReg, Index, Scale};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AddrDispIdx<'a> {
    pub base:  AddrReg,
    pub disp:  Expr<'a>,
    pub index: Index,
    pub scale: Scale,
}

impl<'a> AsmDisplay for AddrDispIdx<'a> {
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle) -> fmt::Result {
        s.write_base_disp_idx(f, &self.base, &self.disp, &self.index, &self.scale)
    }
}

#[cfg(test)]
mod tests {
    use aex::asm::*;
    use aex::ast::Expr;
    use super::*;
    use super::super::{A5, D3, Index, Scale};

    #[test]
    fn display() {
        let x = AddrDispIdx {
            base:  A5,
            disp:  Expr::Int(42),
            index: Index::Data(D3),
            scale: Scale::Word,
        };
        assert_display(&x, &GAS_STYLE, "42(%a5, %d3*2)");
    }
}

