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


use std::io::{self, BufRead};

use aex::fmt::ToCode;
use aex::ast::{Expr, Int};

use super::{AddrReg, DecodeContext, Index, Scale};

/// ColdFire addressing mode: address register indirect with scaled index and
/// displacement.
///
#[derive(Clone, /*PartialEq, Eq, Hash,*/ Debug)]
pub struct AddrDispIdx<'a> {
    /// Base register: an address register.
    pub base:  AddrReg,

    /// 8-bit signed displacement.
    pub disp:  Expr<'a>,

    /// Index register: either an address register or data register.
    pub index: Index,

    /// Index scaling factor.
    pub scale: Scale,
}

impl<'a> AddrDispIdx<'a> {
    /// Decodes an `AddrDispIdx` from the given instruction bits.
    pub fn decode<R: BufRead>(reg: u8, c: &mut DecodeContext<R>) -> io::Result<Self> {
        let ext = c.read_u16()?;

        Ok(AddrDispIdx {
            base:  AddrReg::with_num(reg),
            disp:  Expr::Int(Int::from(ext as u8 as u32)),
            index: Index::decode(ext, 12),
            scale: Scale::decode(ext,  9),
        })
    }
}

impl<'a, A> ToCode<A> for AddrDispIdx<'a> {
    type Output = Expr<'a, A>;

    /// Converts to a code-formattable value with the given annotation.
    #[inline]
    fn to_code(&self, ann: A) -> Self::Output {
        // TODO: Need AST for indirect addressing
        panic!()
    }
}

/*
#[cfg(test)]
mod tests {
    use aex::fmt::*;
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
*/

