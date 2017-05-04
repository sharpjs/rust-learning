// ColdFire Addressing Modes
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

use std::io;

//use aex::fmt::ToCode;
use aex::ast::Expr;
use aex::util::BitPos;

use super::{AddrReg, DataReg, DecodeRead, Scale};

/// ColdFire addressing modes bitfield.
pub type Modes = u16;

pub const DR: Modes = 1 <<  0; // 0.*: data reg direct
pub const AR: Modes = 1 <<  1; // 1.*: addr reg direct
pub const AI: Modes = 1 <<  2; // 2.*: addr reg indirect
pub const AP: Modes = 1 <<  3; // 3.*: addr reg indirect, auto-increment (plus)
pub const AM: Modes = 1 <<  4; // 4.*: addr reg indirect, auto-decrement (minus)
pub const AD: Modes = 1 <<  5; // 5.*: addr reg indirect, displaced
pub const AX: Modes = 1 <<  6; // 6.*: addr reg indirect, indexed, displaced
pub const MS: Modes = 1 <<  7; // 7.0: absolute short
pub const ML: Modes = 1 <<  8; // 7.1: absolute long
pub const PD: Modes = 1 <<  9; // 7.2: pc-relative, displaced
pub const PX: Modes = 1 << 10; // 7.3: pc-relative, indexed, displaced
pub const IM: Modes = 1 << 11; // 7.4: immediate

/// A ColdFire operand location specified by addressing mode.
#[derive(Clone, /*PartialEq, Eq, Hash,*/ Debug)]
pub enum Mode<'a> {
    /// Data register direct mode.
    Data(DataReg),

    /// Address register direct mode.
    Addr(AddrReg),

    /// Address register indirect mode.
    AddrInd(AddrReg),

    /// Address register indirect with post-increment mode.
    AddrPostInc(AddrReg),

    /// Address register indirect with pre-decrement mode.
    AddrPreDec(AddrReg),

    /// Address register indirect with displacement mode.
    AddrDisp(AddrReg, Expr<'a>),

    /// Address register indirect with scaled index and displacement mode.
    AddrIdxDisp(AddrReg, Index, Expr<'a>),

    /// Program counter indirect with displacement mode.
    PcDisp(Expr<'a>),

    /// Program counter indirect with scaled index and displacement mode.
    PcIdxDisp(Index, Expr<'a>),

    /// Absolute short mode (signed 16-bit address).
    Abs16(Expr<'a>),

    /// Absolute long mode (unsigned 32-bit address).
    Abs32(Expr<'a>),

    /// Immediate mode.
    Imm(Expr<'a>),
}

/// A Coldfire scaled index.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Index {
    /// The index register.
    pub reg: IndexReg,

    /// The scaling factor.
    pub scale: Scale,
}

/// ColdFire index registers.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum IndexReg {
    /// Data register used as an index register.
    Data(DataReg),

    /// Address register used as an index register.
    Addr(AddrReg),
}

impl<'a> Mode<'a> {
    /// Decodes a `Mode` from the given instruction bits.
    pub fn decode<R: DecodeRead>(pos: BitPos, c: &mut R) -> io::Result<Self> {
        panic!()
    }
}

/*
impl<'a> Code for Mode<'a> {
    fn fmt(&self, f: &mut Formatter, s: &Style) -> fmt::Result {
        match *self {
            Mode::Data        (ref r) => r.fmt(f, s),
            Mode::Addr        (ref r) => r.fmt(f, s),
            Mode::AddrInd     (ref r) => s.write_ind(f, r),
            Mode::AddrPostInc  (ref r) => s.write_ind_postinc(f, r),
            Mode::AddrPreDec  (ref r) => s.write_ind_predec(f, r),
            Mode::AddrDisp    (ref x) => x.fmt(f, s),
            Mode::AddrIdxDisp (ref x) => x.fmt(f, s),
          //Mode::PcDisp      (ref x) => x.fmt(f, s),
          //Mode::PcIdxDisp   (ref x) => x.fmt(f, s),
            Mode::Abs16       (ref e) => e.fmt(f, s),
            Mode::Abs32       (ref e) => e.fmt(f, s),
            Mode::Imm         (ref e) => e.fmt(f, s),
        }
    }
}

impl<'a> Mode<'a> {
    pub fn decode<R: Read>(word: u16, pos: u8, more: &mut R) -> io::Result<Self> {
        let reg  = (word >> pos     & 7) as u8;
        let mode = (word >> pos + 3 & 7) as u8;
        let size = 2u8; // TODO: parameter

        match (mode, reg, size) {
            (0, _, _) => Ok(Mode::Data(        DataReg::with_num(reg)                   )),
            (1, _, _) => Ok(Mode::Addr(        AddrReg::with_num(reg)                   )),
            (2, _, _) => Ok(Mode::AddrInd(     AddrReg::with_num(reg)                   )),
            (3, _, _) => Ok(Mode::AddrPostInc(  AddrReg::with_num(reg)                   )),
            (4, _, _) => Ok(Mode::AddrPreDec(  AddrReg::with_num(reg)                   )),
            (5, _, _) => Ok(Mode::AddrDisp(    AddrDisp::decode(reg, more)?             )),
            (6, _, _) => Ok(Mode::AddrIdxDisp( AddrIdxDisp::decode(reg, more)?          )),
            (7, 0, _) => Ok(Mode::Abs16(       Expr::Int(more.read_i16::<BE>()? as u32) )),
            (7, 1, _) => Ok(Mode::Abs32(       Expr::Int(more.read_u32::<BE>()?)        )),
          //(7, 2, _) => Ok(Mode::PcDisp(      PcDisp::decode(more)?                    )),
          //(7, 3, _) => Ok(Mode::PcIdxDisp(   PcIdxDisp::decode(more)?                 )),
            (7, 4, 2) => Ok(Mode::Imm(         Expr::Int(more.read_u16::<BE>()? as u32) )),
            (7, 4, 4) => Ok(Mode::Imm(         Expr::Int(more.read_u32::<BE>()? as u32) )),
            _         => invalid()
        }
    }

    pub fn encode(&self, word: &mut u16, pos: u8, more: &mut Vec<u8>) {
        const MASK: u16 = 0x3F;

        let bits: u16 = match *self {
            Mode::Data        (ref r) => (0 << 3) | r.num() as u16,
            Mode::Addr        (ref r) => (1 << 3) | r.num() as u16,
            Mode::AddrInd     (ref r) => (2 << 3) | r.num() as u16,
            Mode::AddrPostInc  (ref r) => (3 << 3) | r.num() as u16,
            Mode::AddrPreDec  (ref r) => (4 << 3) | r.num() as u16,

            Mode::AddrDisp(ref x) => {
                let disp = match x.disp {
                    Expr::Int(n) => n as u16, // TODO: Limit
                    _ => panic!("Non-integer displacement."),
                };
                more.write_u16::<BE>(disp).unwrap();
                (5 << 3) | x.base.num() as u16
            },
            Mode::AddrIdxDisp(ref x) => {
                let disp = match x.disp {
                    Expr::Int(n) => n as u8, // TODO: Limit
                    _ => panic!("Non-integer displacement."),
                };
                // stub
                (6 << 3)
            },
            Mode::Abs16(ref e) => {
                let addr = match *e {
                    Expr::Int(n) => n as u16, // TODO: Limit
                    _ => panic!("Non-integer displacement."),
                };
                more.write_u16::<BE>(addr).unwrap();
                (7 << 3) | 0
            },
            Mode::Abs32(ref e) => {
                let addr = match *e {
                    Expr::Int(n) => n, // TODO: Limit
                    _ => panic!("Non-integer displacement."),
                };
                more.write_u32::<BE>(addr).unwrap();
                (7 << 3) | 0
            },
          //Mode::PcDisp(ref x) => {
          //    let disp = match x.disp {
          //        Expr::Int(n) => n as u16, // TODO: Limit
          //        _ => panic!("Non-integer displacement."),
          //    };
          //    more.write_u16::<BE>(disp).unwrap();
          //    (7 << 3) | 2
          //},
          //Mode::PcIdxDisp(ref x) => {
          //    let disp = match x.disp {
          //        Expr::Int(n) => n as u8, // TODO: Limit
          //        _ => panic!("Non-integer displacement."),
          //    };
          //    // stub
          //    (7 << 3) | 3
          //},
            _ => 0
        };

        *word = *word & (MASK << pos) | (bits << pos);
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;
    use super::super::{AddrDisp, AddrIdxDisp, D3, FP, Index, Scale}; 
    use aex::fmt::*;
    use aex::ast::Expr;

    #[test]
    fn display_data_reg() {
        let value  = Mode::Data(D3);
        assert_display(&value, &GAS_STYLE, "%d3");
    }

    #[test]
    fn display_addr_reg() {
        let value  = Mode::Addr(FP);
        assert_display(&value, &GAS_STYLE, "%fp");
    }

    #[test]
    fn display_addr_reg_ind() {
        let value = Mode::AddrInd(FP);
        assert_display(&value, &GAS_STYLE, "(%fp)");
    }

    #[test]
    fn display_addr_reg_ind_dec() {
        let value = Mode::AddrPreDec(FP);
        assert_display(&value, &GAS_STYLE, "-(%fp)");
    }

    #[test]
    fn display_addr_reg_ind_inc() {
        let value = Mode::AddrPostInc(FP);
        assert_display(&value, &GAS_STYLE, "(%fp)+");
    }

    #[test]
    fn decode_data() {
        let mut more = Cursor::new(vec![]);
        let value = Mode::decode(0b_000_011_00000, 5, &mut more).unwrap();
        assert_eq!(value, Mode::Data(D3));
    }

    #[test]
    fn decode_addr() {
        let mut more = Cursor::new(vec![]);
        let value = Mode::decode(0b_001_110_00000, 5, &mut more).unwrap();
        assert_eq!(value, Mode::Addr(FP));
    }

    #[test]
    fn decode_addr_ind() {
        let mut more = Cursor::new(vec![]);
        let value = Mode::decode(0b_010_110_00000, 5, &mut more).unwrap();
        assert_eq!(value, Mode::AddrInd(FP));
    }

    #[test]
    fn decode_addr_ind_inc() {
        let mut more = Cursor::new(vec![]);
        let value = Mode::decode(0b_011_110_00000, 5, &mut more).unwrap();
        assert_eq!(value, Mode::AddrPostInc(FP));
    }

    #[test]
    fn decode_addr_ind_dec() {
        let mut more = Cursor::new(vec![]);
        let value = Mode::decode(0b_100_110_00000, 5, &mut more).unwrap();
        assert_eq!(value, Mode::AddrPreDec(FP));
    }

    #[test]
    fn decode_addr_disp() {
        let mut more = Cursor::new(vec![0x01, 0x23]);
        let value = Mode::decode(0b_101_110_00000, 5, &mut more).unwrap();
        assert_eq!(value, Mode::AddrDisp(AddrDisp {
            base: FP,
            disp: Expr::Int(0x0123)
        }));
    }

    #[test]
    fn decode_addr_disp_idx() {
        let mut more = Cursor::new(vec![0b0011_1100, 0x12]);
        let value = Mode::decode(0b_110_110_00000, 5, &mut more).unwrap();
        assert_eq!(value, Mode::AddrIdxDisp(AddrIdxDisp {
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
        Mode::Data(D3).encode(&mut word, 5, &mut more);
        assert_eq!(word, 0b_000_011_00000);
        assert_eq!(more, vec![]);
    }

    #[test]
    fn encode_addr() {
        let mut word = 0;
        let mut more = vec![];
        Mode::Addr(FP).encode(&mut word, 5, &mut more);
        assert_eq!(word, 0b_001_110_00000);
        assert_eq!(more, vec![]);
    }

    #[test]
    fn encode_addr_ind() {
        let mut word = 0;
        let mut more = vec![];
        Mode::AddrInd(FP).encode(&mut word, 5, &mut more);
        assert_eq!(word, 0b_010_110_00000);
        assert_eq!(more, vec![]);
    }

    #[test]
    fn encode_addr_ind_inc() {
        let mut word = 0;
        let mut more = vec![];
        Mode::AddrPostInc(FP).encode(&mut word, 5, &mut more);
        assert_eq!(word, 0b_011_110_00000);
        assert_eq!(more, vec![]);
    }

    #[test]
    fn encode_addr_ind_dec() {
        let mut word = 0;
        let mut more = vec![];
        Mode::AddrPreDec(FP).encode(&mut word, 5, &mut more);
        assert_eq!(word, 0b_100_110_00000);
        assert_eq!(more, vec![]);
    }

    #[test]
    fn encode_addr_disp() {
        let mut word = 0;
        let mut more = vec![];
        let value = Mode::AddrDisp(AddrDisp {
            base: FP,
            disp: Expr::Int(0x0123),
        });
        value.encode(&mut word, 5, &mut more);
        assert_eq!(word, 0b_101_110_00000);
        assert_eq!(more, vec![0x01, 0x23]);
    }
}
*/

