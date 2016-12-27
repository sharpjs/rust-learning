// ColdFire Values
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
use super::addr_reg::AddrReg;
use super::data_reg::DataReg;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Value /*<'a>*/ {
    Data        (DataReg),          // Data register
    Addr        (AddrReg),          // Address register
    AddrInd     (AddrReg),          // Address register indirect
    AddrIndDec  (AddrReg),          // Address register indirect, pre-decrement
    AddrIndInc  (AddrReg),          // Address register indirect, post-increment
  //AddrDisp    (AddrDisp   <'a>),  // Address register indirect, displaced
  //AddrDispIdx (AddrDispIdx<'a>),  // Address register indirect, displaced, indexed
  //PcDisp      (PcDisp     <'a>),  // PC-relative, displaced
  //PcDispIdx   (PcDispIdx  <'a>),  // PC-relative, displaced, indexed
  //Abs16       (Expr<'a>),         // Absolute 16-bit value
  //Abs32       (Expr<'a>),         // Absolute 32-bit value
  //Imm         (Expr<'a>),         // Immediate
}

impl AsmDisplay for Value {
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle) -> fmt::Result {
        match *self {
            Value::Data        (ref r) => r.fmt(f, s),
            Value::Addr        (ref r) => r.fmt(f, s),
            Value::AddrInd     (ref r) => s.write_ind(f, r),
            Value::AddrIndDec  (ref r) => s.write_ind_predec(f, r),
            Value::AddrIndInc  (ref r) => s.write_ind_postinc(f, r),
          //Value::AddrDisp    (ref x) => Asm(r, s).fmt(f),
          //Value::AddrDispIdx (ref x) => Asm(r, s).fmt(f),
          //Value::PcDisp      (ref x) => Asm(r, s).fmt(f),
          //Value::PcDispIdx   (ref x) => Asm(r, s).fmt(f),
          //Value::Abs16       (ref e) => Asm(r, s).fmt(f),
          //Value::Abs32       (ref e) => Asm(r, s).fmt(f),
          //Value::Imm         (ref e) => Asm(r, s).fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{D3, FP};
    use aex::asm::*;

    #[test]
    fn display_data_reg() {
        let value  = Value::Data(D3);
        assert_display(&value, &GAS_STYLE, "%d3");
    }

    #[test]
    fn display_addr_reg() {
        let value  = Value::Addr(FP);
        assert_display(&value, &GAS_STYLE, "%fp");
    }

    #[test]
    fn display_addr_reg_ind() {
        let value = Value::AddrInd(FP);
        assert_display(&value, &GAS_STYLE, "(%fp)");
    }

    #[test]
    fn display_addr_reg_ind_dec() {
        let value = Value::AddrIndDec(FP);
        assert_display(&value, &GAS_STYLE, "-(%fp)");
    }

    #[test]
    fn display_addr_reg_ind_inc() {
        let value = Value::AddrIndInc(FP);
        assert_display(&value, &GAS_STYLE, "(%fp)+");
    }
}

