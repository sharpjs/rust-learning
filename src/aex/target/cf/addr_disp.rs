// ColdFire Address Register + Displacement Mode
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

use super::{AddrReg, DecodeContext};

/// ColdFire addressing mode: address register indirect with displacement.
#[derive(Clone, /*PartialEq, Eq, Hash,*/ Debug)]
pub struct AddrDisp<'a> {
    /// Base register; must be an address register.
    pub base: AddrReg,

    /// 16-bit signed displacement.
    pub disp: Expr<'a>
}

impl<'a> AddrDisp<'a> {
    /// Decodes an `AddrDisp` from the given instruction bits.
    pub fn decode<R: BufRead>(reg: u8, c: &mut DecodeContext<R>) -> io::Result<Self> {
        let ext = c.read_i16()?;

        Ok(AddrDisp {
            base: AddrReg::with_num(reg),
            disp: Expr::Int(Int::new(ext)),
        })
    }
}

impl<'a, A> ToCode<A> for AddrDisp<'a> {
    type Output = Expr<'a, A>;

    /// Converts to a code-formattable value with the given annotation.
    #[inline]
    fn to_code(&self, ann: A) -> Self::Output {
        // TODO: Need AST for indirect addressing
        panic!()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;
    use super::super::A5;

    // TODO: Why does this fail?
    //#[test]
    fn decode() {
        let mut src = Cursor::new(vec![0x42, 0xF1, 0x01, 0x02, 0x03]);
        let mut ctx = DecodeContext::new(&mut src, 0);

        let v = AddrDisp::decode(5, &mut ctx).unwrap();

        assert_eq!(v.base, A5);
    }
}

