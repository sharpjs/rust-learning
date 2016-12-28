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
    Data        (DataReg),          // Data    register
    Addr        (AddrReg),          // Address register
    AddrInd     (AddrReg),          // Address register indirect
    AddrIndDec  (AddrReg),          // Address register indirect, pre-decrement
    AddrIndInc  (AddrReg),          // Address register indirect, post-increment
  //AddrDisp    (AddrDisp   <'a>),  // Address register indirect, displaced
  //AddrDispIdx (AddrDispIdx<'a>),  // Address register indirect, displaced, indexed
  //PcDisp      (PcDisp     <'a>),  // PC-relative, displaced
  //PcDispIdx   (PcDispIdx  <'a>),  // PC-relative, displaced, indexed
  //Abs16       (Expr<'a>),         // Absolute, signed   16-bit address
  //Abs32       (Expr<'a>),         // Absolute, unsigned 32-bit address
  //Imm         (Expr<'a>),         // Immediate (variable bit width)
}

impl AsmDisplay for Value {
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle) -> fmt::Result {
        match *self {
            Value::Data        (ref r) => r.fmt(f, s),
            Value::Addr        (ref r) => r.fmt(f, s),
            Value::AddrInd     (ref r) => s.write_ind(f, r),
            Value::AddrIndInc  (ref r) => s.write_ind_postinc(f, r),
            Value::AddrIndDec  (ref r) => s.write_ind_predec(f, r),
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

impl Value {
    pub fn decode(word: u16, pos: u8) -> Option<Self> {
        let reg  = (word >> pos     & 7) as u8;
        let mode = (word >> pos + 3 & 7) as u8;

        match mode {
            0 => Some(Value::Data       (DataReg::with_num(reg))),
            1 => Some(Value::Addr       (AddrReg::with_num(reg))),
            2 => Some(Value::AddrInd    (AddrReg::with_num(reg))),
            3 => Some(Value::AddrIndInc (AddrReg::with_num(reg))),
            4 => Some(Value::AddrIndDec (AddrReg::with_num(reg))),
            _ => None,
        }
    }

    pub fn encode(&self, word: &mut u16, pos: u8) {
        const MASK: u16 = 0x3F;

        let bits: u16 = match *self {
            Value::Data        (ref r) => (0 << 3) | r.num() as u16,
            Value::Addr        (ref r) => (1 << 3) | r.num() as u16,
            Value::AddrInd     (ref r) => (2 << 3) | r.num() as u16,
            Value::AddrIndInc  (ref r) => (3 << 3) | r.num() as u16,
            Value::AddrIndDec  (ref r) => (4 << 3) | r.num() as u16,
        };

        *word = *word & (MASK << pos) | (bits << pos);
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

    #[test]
    fn decode_data() {
        let value = Value::decode(0b0000_0000_0110_0000, 5);
        assert_eq!(value, Some(Value::Data(D3)));
    }

    #[test]
    fn decode_addr() {
        let value = Value::decode(0b0000_0001_1100_0000, 5);
        assert_eq!(value, Some(Value::Addr(FP)));
    }

    #[test]
    fn decode_addr_ind() {
        let value = Value::decode(0b0000_0010_1100_0000, 5);
        assert_eq!(value, Some(Value::AddrInd(FP)));
    }

    #[test]
    fn decode_addr_ind_inc() {
        let value = Value::decode(0b0000_0011_1100_0000, 5);
        assert_eq!(value, Some(Value::AddrIndInc(FP)));
    }

    #[test]
    fn decode_addr_ind_dec() {
        let value = Value::decode(0b0000_0100_1100_0000, 5);
        assert_eq!(value, Some(Value::AddrIndDec(FP)));
    }

    #[test]
    fn encode_data() {
        let mut word: u16 = 0;
        Value::Data(D3).encode(&mut word, 5);
        assert_eq!(word, 0b0000_0000_0110_0000);
    }

    #[test]
    fn encode_addr() {
        let mut word: u16 = 0;
        Value::Addr(FP).encode(&mut word, 5);
        assert_eq!(word, 0b0000_0001_1100_0000);
    }

    #[test]
    fn encode_addr_ind() {
        let mut word: u16 = 0;
        Value::AddrInd(FP).encode(&mut word, 5);
        assert_eq!(word, 0b0000_0010_1100_0000);
    }

    #[test]
    fn encode_addr_ind_inc() {
        let mut word: u16 = 0;
        Value::AddrIndInc(FP).encode(&mut word, 5);
        assert_eq!(word, 0b0000_0011_1100_0000);
    }

    #[test]
    fn encode_addr_ind_dec() {
        let mut word: u16 = 0;
        Value::AddrIndDec(FP).encode(&mut word, 5);
        assert_eq!(word, 0b0000_0100_1100_0000);
    }
}

