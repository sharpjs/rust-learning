// ColdFire Values
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
use byteorder::{BigEndian as BE, ReadBytesExt, WriteBytesExt};

use aex::asm::{AsmDisplay, AsmStyle};
use aex::ast::Expr;
use aex::util::invalid;

use super::{AddrDisp, AddrDispIdx, AddrReg, DataReg, Index, Scale};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Value<'a> {
    Data        (DataReg),          // Data    register
    Addr        (AddrReg),          // Address register
    AddrInd     (AddrReg),          // Address register indirect
    AddrIndDec  (AddrReg),          // Address register indirect, pre-decrement
    AddrIndInc  (AddrReg),          // Address register indirect, post-increment
    AddrDisp    (AddrDisp   <'a>),  // Address register indirect, displaced
    AddrDispIdx (AddrDispIdx<'a>),  // Address register indirect, displaced, indexed
  //PcDisp      (PcDisp     <'a>),  // PC-relative, displaced
  //PcDispIdx   (PcDispIdx  <'a>),  // PC-relative, displaced, indexed
  //Abs16       (Expr<'a>),         // Absolute, signed   16-bit address
  //Abs32       (Expr<'a>),         // Absolute, unsigned 32-bit address
  //Imm         (Expr<'a>),         // Immediate (variable bit width)
}

impl<'a> AsmDisplay for Value<'a> {
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle) -> fmt::Result {
        match *self {
            Value::Data        (ref r) => r.fmt(f, s),
            Value::Addr        (ref r) => r.fmt(f, s),
            Value::AddrInd     (ref r) => s.write_ind(f, r),
            Value::AddrIndInc  (ref r) => s.write_ind_postinc(f, r),
            Value::AddrIndDec  (ref r) => s.write_ind_predec(f, r),
            Value::AddrDisp    (ref x) => x.fmt(f, s),
            Value::AddrDispIdx (ref x) => x.fmt(f, s),
          //Value::PcDisp      (ref x) => x.fmt(f, s),
          //Value::PcDispIdx   (ref x) => x.fmt(f, s),
          //Value::Abs16       (ref e) => e.fmt(f, s),
          //Value::Abs32       (ref e) => e.fmt(f, s),
          //Value::Imm         (ref e) => e.fmt(f, s),
        }
    }
}

impl<'a> Value<'a> {
    pub fn decode<R: Read>(word: u16, pos: u8, more: &mut R) -> io::Result<Self> {
        let reg  = (word >> pos     & 7) as u8;
        let mode = (word >> pos + 3 & 7) as u8;

        match mode {
            0 => Ok(Value::Data(       DataReg::with_num(reg) )),
            1 => Ok(Value::Addr(       AddrReg::with_num(reg) )),
            2 => Ok(Value::AddrInd(    AddrReg::with_num(reg) )),
            3 => Ok(Value::AddrIndInc( AddrReg::with_num(reg) )),
            4 => Ok(Value::AddrIndDec( AddrReg::with_num(reg) )),
            5 => {
                let ext = more.read_u16::<BE>()?;

                Ok(Value::AddrDisp(
                    AddrDisp {
                        base: AddrReg::with_num(reg),
                        disp: Expr::Int(ext as u32)
                    }
                ))
            },
            6 => {
                let ext = more.read_u16::<BE>()?;

                Ok(Value::AddrDispIdx(
                    AddrDispIdx {
                        base:  AddrReg::with_num(reg),
                        disp:  Expr::Int(ext as u8 as u32),
                        index: Index::decode(ext, 12),
                        scale: Scale::decode(ext, 9)?,
                    }
                ))
            },
            _ => invalid(),
        }
    }

    pub fn encode(&self, word: &mut u16, pos: u8, more: &mut Vec<u8>) {
        const MASK: u16 = 0x3F;

        let bits: u16 = match *self {
            Value::Data        (ref r) => (0 << 3) | r.num() as u16,
            Value::Addr        (ref r) => (1 << 3) | r.num() as u16,
            Value::AddrInd     (ref r) => (2 << 3) | r.num() as u16,
            Value::AddrIndInc  (ref r) => (3 << 3) | r.num() as u16,
            Value::AddrIndDec  (ref r) => (4 << 3) | r.num() as u16,

            Value::AddrDisp(ref x) => {
                let disp = match x.disp {
                    Expr::Int(n) => n as u16,
                    _ => panic!("Non-integer displacement."),
                };
                more.write_u16::<BE>(disp).unwrap();
                (5 << 3) | x.base.num() as u16
            },
            Value::AddrDispIdx(ref x) => {
                // stub
                (6 << 3)
            },
        };

        *word = *word & (MASK << pos) | (bits << pos);
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;
    use super::super::*;
    use aex::asm::*;
    use aex::ast::Expr;

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
        let mut more = Cursor::new(vec![]);
        let value = Value::decode(0b0000_0000_0110_0000, 5, &mut more).unwrap();
        assert_eq!(value, Value::Data(D3));
    }

    #[test]
    fn decode_addr() {
        let mut more = Cursor::new(vec![]);
        let value = Value::decode(0b0000_0001_1100_0000, 5, &mut more).unwrap();
        assert_eq!(value, Value::Addr(FP));
    }

    #[test]
    fn decode_addr_ind() {
        let mut more = Cursor::new(vec![]);
        let value = Value::decode(0b0000_0010_1100_0000, 5, &mut more).unwrap();
        assert_eq!(value, Value::AddrInd(FP));
    }

    #[test]
    fn decode_addr_ind_inc() {
        let mut more = Cursor::new(vec![]);
        let value = Value::decode(0b0000_0011_1100_0000, 5, &mut more).unwrap();
        assert_eq!(value, Value::AddrIndInc(FP));
    }

    #[test]
    fn decode_addr_ind_dec() {
        let mut more = Cursor::new(vec![]);
        let value = Value::decode(0b0000_0100_1100_0000, 5, &mut more).unwrap();
        assert_eq!(value, Value::AddrIndDec(FP));
    }

    #[test]
    fn decode_addr_disp() {
        let mut more = Cursor::new(vec![0x01, 0x23]);
        let value = Value::decode(0b0000_0101_1100_0000, 5, &mut more).unwrap();
        assert_eq!(value, Value::AddrDisp(AddrDisp {
            base: FP,
            disp: Expr::Int(0x0123)
        }));
    }

    #[test]
    fn decode_addr_disp_idx() {
        let mut more = Cursor::new(vec![0b0011_1100, 0x12]);
        let value = Value::decode(0b0000_0110_1100_0000, 5, &mut more).unwrap();
        assert_eq!(value, Value::AddrDispIdx(AddrDispIdx {
            base:  FP,
            disp:  Expr::Int(0x12),
            index: Index::Data(D3),
            scale: Scale::Long,
        }));
    }

    #[test]
    fn encode_data() {
        let mut word = 0;
        let mut more = vec![];
        Value::Data(D3).encode(&mut word, 5, &mut more);
        assert_eq!(word, 0b0000_0000_0110_0000);
        assert_eq!(more, vec![]);
    }

    #[test]
    fn encode_addr() {
        let mut word = 0;
        let mut more = vec![];
        Value::Addr(FP).encode(&mut word, 5, &mut more);
        assert_eq!(word, 0b0000_0001_1100_0000);
        assert_eq!(more, vec![]);
    }

    #[test]
    fn encode_addr_ind() {
        let mut word = 0;
        let mut more = vec![];
        Value::AddrInd(FP).encode(&mut word, 5, &mut more);
        assert_eq!(word, 0b0000_0010_1100_0000);
        assert_eq!(more, vec![]);
    }

    #[test]
    fn encode_addr_ind_inc() {
        let mut word = 0;
        let mut more = vec![];
        Value::AddrIndInc(FP).encode(&mut word, 5, &mut more);
        assert_eq!(word, 0b0000_0011_1100_0000);
        assert_eq!(more, vec![]);
    }

    #[test]
    fn encode_addr_ind_dec() {
        let mut word = 0;
        let mut more = vec![];
        Value::AddrIndDec(FP).encode(&mut word, 5, &mut more);
        assert_eq!(word, 0b0000_0100_1100_0000);
        assert_eq!(more, vec![]);
    }

    #[test]
    fn encode_addr_disp() {
        let mut word = 0;
        let mut more = vec![];
        let value = Value::AddrDisp(AddrDisp {
            base: FP,
            disp: Expr::Int(0x0123),
        });
        value.encode(&mut word, 5, &mut more);
        assert_eq!(word, 0b0000_0101_1100_0000);
        assert_eq!(more, vec![0x01, 0x23]);
    }
}

