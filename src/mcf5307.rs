// MCF5307 Target
//
// This file is part of AEx.
// Copyright (C) 2015 Jeffrey Sharp
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

#![allow(non_upper_case_globals)]

use std::borrow::Borrow;
use std::fmt::{self, Display, Formatter, Write};
use std::ops::BitOr;
use std::io;

use ast::Expr;
use types::*;
use util::*;

// Locations - place where data can be read or written

trait Loc : LocEq + Display {
    fn mode(&self) -> ModeId;
    fn is_q(&self) -> bool { false }
}

derive_dynamic_eq!(Loc : LocEq);

// Constants (defined in types.rs)

impl Display for Const {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Const::Sym(ref s)           => s.fmt(f),
            Const::Num(    v) if v < 10 => write!(f, "{}",    v),
            Const::Num(    v)           => write!(f, "{:#X}", v),
        }
    }
}

// Immediate

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Imm_ (Const);

impl Loc for Imm_ {
    fn mode(&self) -> ModeId { M_Imm }
    fn is_q(&self) -> bool {
        match self.0 {
            Const::Num(i) => 1 <= i && i <= 8,
            _             => false
        }
    }
}

impl Display for Imm_ {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "#{}", &self.0)
    }
}

// Data Registers

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(u8)]
pub enum DataReg { D0, D1, D2, D3, D4, D5, D6, D7 }
use self::DataReg::*;

pub static DATA_REGS: [DataReg; 8] = [D0, D1, D2, D3, D4, D5, D6, D7];

impl DataReg {
    fn num(self) -> u8 { self as u8 }
}

impl Display for DataReg {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%d{}", *self as u8)
    }
}

// Address Registers

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(u8)]
pub enum AddrReg { A0, A1, A2, A3, A4, A5, A6, A7 }
use self::AddrReg::*;

pub static ADDR_REGS: [AddrReg; 8] = [A0, A1, A2, A3, A4, A5, A6, A7];

impl AddrReg {
    fn num(self) -> u8 { self as u8 }
}

impl Display for AddrReg {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%a{}", *self as u8)
    }
}

// Register Set

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct RegSet (u16);

// This is a bitmask of the numbered registers:
//   bits: [15 .. 08] [07 .. 00]
//   regs:  a7 .. a0   d7 .. d0

impl From<DataReg> for RegSet {
    fn from(r: DataReg) -> RegSet { RegSet(0x0001 << r.num()) }
}

impl From<AddrReg> for RegSet {
    fn from(r: AddrReg) -> RegSet { RegSet(0x0100 << r.num()) }
}

impl<R: Into<RegSet>> BitOr<R> for DataReg {
    type Output = RegSet;
    fn bitor(self, r: R) -> RegSet { RegSet::from(self) | r.into() }
}

impl<R: Into<RegSet>> BitOr<R> for AddrReg {
    type Output = RegSet;
    fn bitor(self, r: R) -> RegSet { RegSet::from(self) | r.into() }
}

impl<R: Into<RegSet>> BitOr<R> for RegSet {
    type Output = RegSet;
    fn bitor(self, r: R) -> RegSet { RegSet(self.0 | r.into().0) }
}

impl Display for RegSet {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let join =
        try!(fmt_regs((self.0 & 0xFF) as u8, &DATA_REGS, false, f));
        try!(fmt_regs((self.0 >>   8) as u8, &ADDR_REGS, join,  f));
        Ok(())
    }
}

fn fmt_regs<R>(bits: u8, regs: &[R; 8], mut join: bool, f: &mut Formatter)
    -> Result<bool, fmt::Error>
    where R: Display
{
    let mut n     = 0;      // register number
    let mut bit   = 1;      // bit for register in bitmask
    let mut start = None;   // register number starting current range

    loop {
        let has = n < 8 && (bits & bit) != 0;

        match (has, start) {
            (true, None) => {
                start = Some(n)
            },
            (false, Some(s)) => {
                if join {
                    try!(f.write_char('/'))
                }
                try!(regs[s].fmt(f));
                if n > s + 1 {
                    try!(f.write_char('-'));
                    try!(regs[n - 1].fmt(f));
                }
                start = None;
                join  = true;
            },
            _ => { /*nop*/ }
        }

        if n == 8 { break }
        n += 1;
        bit = bit.wrapping_shl(1);
    }

    Ok(join)
}

// Control Registers

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CtrlReg { VBR, CACR, ACR0, ACR1, MBAR, RAMBAR }

impl Display for CtrlReg {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let s = match *self {
            CtrlReg::VBR    => "%vbr",
            CtrlReg::CACR   => "%cacr",
            CtrlReg::ACR0   => "%acr0",
            CtrlReg::ACR1   => "%acr1",
            CtrlReg::MBAR   => "%mbar",
            CtrlReg::RAMBAR => "%rambar",
        };
        f.write_str(s)
    }
}

// Index Registers

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum Index {
    Data (DataReg),
    Addr (AddrReg),
}

impl Display for Index {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Index::Data(ref r) => r.fmt(f),
            Index::Addr(ref r) => r.fmt(f),
        }
    }
}

// Addressing Modes

// NOTE: Required because stable Rust does not provide an API to read enum
// discriminator values.

// NOTE: Cannot use newtype pattern here, as Rust does not have "generalized
// newtype deriving" like Haskell.  We would want ModeId to derive BitOr.

type ModeId = u32;
const M_Imm:         ModeId = 1 <<  0;
const M_Abs16:       ModeId = 1 <<  1;
const M_Abs32:       ModeId = 1 <<  2;
const M_Data:        ModeId = 1 <<  3;
const M_Addr:        ModeId = 1 <<  4;
const M_Ctrl:        ModeId = 1 <<  5;
const M_Regs:        ModeId = 1 <<  6;
const M_AddrInd:     ModeId = 1 <<  7;
const M_AddrIndInc:  ModeId = 1 <<  8;
const M_AddrIndDec:  ModeId = 1 <<  9;
const M_AddrDisp:    ModeId = 1 << 10;
const M_AddrDispIdx: ModeId = 1 << 11;
const M_PcDisp:      ModeId = 1 << 12;
const M_PcDispIdx:   ModeId = 1 << 13;
const M_PC:          ModeId = 1 << 14;
const M_SR:          ModeId = 1 << 15;
const M_CCR:         ModeId = 1 << 16;
const M_BC:          ModeId = 1 << 17;

const M_Reg: ModeId
    = M_Data | M_Addr;

const M_Dst: ModeId
    = M_Reg | M_AddrInd | M_AddrIndInc | M_AddrIndDec | M_AddrDisp | M_AddrDispIdx;

const M_Src: ModeId
    = M_Dst | M_Imm | M_PcDisp | M_PcDispIdx;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Mode {
    // Immediate/Absolute
    Imm         (Const),
    Abs16       (Const),
    Abs32       (Const),

    // Direct
    Data        (DataReg),
    Addr        (AddrReg),
    Ctrl        (CtrlReg),
    Regs        (RegSet),

    // Indirect
    AddrInd     (AddrReg),
    AddrIndInc  (AddrReg),
    AddrIndDec  (AddrReg),
    AddrDisp    (AddrReg, Const),
    AddrDispIdx (AddrReg, Const, Index),
    PcDisp      (         Const),
    PcDispIdx   (         Const, Index),

    // Special
    PC, SR, CCR, BC,
}
use self::Mode::*;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Operand<'a> {
    pos:    Pos,
    ty:     &'a Type,
    mode:   Mode
}

impl Mode {
    fn id(&self) -> ModeId {
        match *self {
            Imm         (..) => M_Imm,
            Abs16       (..) => M_Abs16,
            Abs32       (..) => M_Abs32,
            Data        (..) => M_Data,
            Addr        (..) => M_Addr,
            Ctrl        (..) => M_Ctrl,
            AddrInd     (..) => M_AddrInd,
            AddrIndInc  (..) => M_AddrIndInc,
            AddrIndDec  (..) => M_AddrIndDec,
            AddrDisp    (..) => M_AddrDisp,
            AddrDispIdx (..) => M_AddrDispIdx,
            PcDisp      (..) => M_PcDisp,
            PcDispIdx   (..) => M_PcDispIdx,
            Regs        (..) => M_Regs,
            PC               => M_PC,
            SR               => M_SR,
            CCR              => M_CCR,
            BC               => M_BC,
        }
    }

    fn is(&self, modes: ModeId) -> bool {
        self.id() & modes != 0
    }

    fn is_q(&self) -> bool {
        match *self {
            Imm(Const::Num(i)) => 1 <= i && i <= 8,
            _ => false
        }
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Imm         (ref v)               => write!(f, "#{}",  v),
            Abs16       (ref v)               => write!(f, "{}:w", v),
            Abs32       (ref v)               => write!(f, "{}:l", v),
            Data        (ref r)               => r.fmt(f),
            Addr        (ref r)               => r.fmt(f),
            Ctrl        (ref r)               => r.fmt(f),
            Regs        (ref r)               => r.fmt(f),
            AddrInd     (ref r)               => write!(f, "({})",  r),
            AddrIndInc  (ref r)               => write!(f, "({})+", r),
            AddrIndDec  (ref r)               => write!(f, "-({})", r),
            AddrDisp    (ref b, ref d)        => write!(f, "({},{})",       b, d),
            AddrDispIdx (ref b, ref d, ref i) => write!(f, "({},{},{}*{})", b, d, i, 1),
            PcDisp      (       ref d)        => write!(f, "(%pc,{})",       d),
            PcDispIdx   (       ref d, ref i) => write!(f, "(%pc,{},{}*{})", d, i, 1),
            PC                                => f.write_str("%pc"),
            SR                                => f.write_str("%sr"),
            CCR                               => f.write_str("%ccr"),
            BC                                => f.write_str("bc"),
        }
    }
}

// Code Generator

pub struct CodeGen<W: io::Write> {
    out: W
}

impl<W> CodeGen<W> where W: io::Write {
    pub fn new(out: W) -> Self {
        CodeGen { out: out }
    }

    // This is all WIP, just idea exploration.

    pub fn visit_expr<'a>(&mut self, e: &Expr) -> Box<Operand<'a>> {
        match *e {
            Expr::Add(ref src, ref dst, sel) => {
                let src = self.visit_expr(src);
                let dst = self.visit_expr(dst);
                // TODO: interpret sel
                self.add_g(src, dst)
            },
            Expr::Int(n) => {
                // TODO: This needs to be of an "indeterminate integer" type
                Box::new(Operand { pos: Pos::bof(), ty: U64, mode: Imm(Const::Num(n)) })
            }
            _ => {
                panic!("not supported yet");
            }
        }
    }

    pub fn add_g<'s, 'd, S, D>(&mut self, src: S, dst: D) -> D
        where S: Borrow<Operand<'s>>,
              D: Borrow<Operand<'d>>
    {
        {
            let s = src.borrow();
            let d = dst.borrow();
            require_types_equal(s, d);

            let s = &s.mode;
            let d = &d.mode;
            match (s.id(), d.id()) {
                (M_Data, _) if d.is(M_Dst) => self.write_insn_2("add", s, d),
                (_, M_Data) if s.is(M_Src) => self.write_insn_2("add", s, d),
                _                          => panic!("X")
            }
        }
        dst
    }

    fn add_test(&mut self, src: &Loc, dst: &Loc) {
        let sm = src.mode();
        let dm = dst.mode();
        match (sm, dm) {
            (M_Imm, M_Imm) => { src.as_any().downcast_ref::<Imm_>(); },
            _ => {}
        }
    }

    fn write_insn_2(&mut self, op: &str, src: &Mode, dst: &Mode) {
        writeln!(self.out, "    {} {}, {}", op, src, dst).unwrap();
    }
}

fn require_types_equal(a: &Operand, b: &Operand)
{
    if &a.ty != &b.ty {
        panic!("Type mismatch."); // TODO: Error
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Mode::*;
    use super::DataReg::*;
    use super::AddrReg::*;

    use std::io;

    use types::*;
    use util::*;

    #[test]
    fn foo() {
        let src = Box::new(Operand { pos: Pos::bof(), ty: U8, mode: Imm(Const::Num(4)) });
        let dst = Box::new(Operand { pos: Pos::bof(), ty: U8, mode: Data(D0) });
        let mut gen = CodeGen::new(io::stdout());
        let res = gen.add_g(src, dst.clone());
        assert_eq!(dst, res);
    }

    #[test]
    fn fmt_regs() {
        let s = format!("{}", Regs(D0 | D3 | D6 | D7 | A1 | A2 | A3));
        assert_eq!(s, "%d0/%d3/%d6-%d7/%a1-%a3");
    }
}

