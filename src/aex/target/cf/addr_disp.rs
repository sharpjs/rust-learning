// ColdFire Address Register + Displacement Mode
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
use aex::ast::Expr;
use super::AddrReg;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AddrDisp<'a> {
    pub base: AddrReg,
    pub disp: Expr<'a>
}

impl<'a> AsmDisplay for AddrDisp<'a> {
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle) -> fmt::Result {
        let disp = *self.disp.0;          // TODO: Asm(&self.disp, s);
        s.write_base_disp(f, &self.base, &disp)
    }
}

#[cfg(test)]
mod tests {
    use aex::asm::*;
    use aex::ast::Expr;
    use super::*;
    use super::super::A5;

    #[test]
    fn display() {
        let d = 42; // TODO
        let x = AddrDisp { base: A5, disp: Expr(&d) };
        assert_display(&x, &GAS_STYLE, "42(%a5)");
    }
}

