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
use std::fmt::{self, Display, Formatter, Write as WriteFmt};
use std::io::{self, Write};

use types::*;
use util::*;

// Addressing Mode Id

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

// Data Registers

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct DataReg (u8);

pub const D0: DataReg = DataReg(0);
pub const D1: DataReg = DataReg(1);
pub const D2: DataReg = DataReg(2);
pub const D3: DataReg = DataReg(3);
pub const D4: DataReg = DataReg(4);
pub const D5: DataReg = DataReg(5);
pub const D6: DataReg = DataReg(6);
pub const D7: DataReg = DataReg(7);

static DATA_REGS: [DataReg; 8] = [D0, D1, D2, D3, D4, D5, D6, D7];

impl Display for DataReg {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%d{}", self.0)
    }
}

// Address Registers

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct AddrReg (u8);

pub const A0: AddrReg = AddrReg(0);
pub const A1: AddrReg = AddrReg(1);
pub const A2: AddrReg = AddrReg(2);
pub const A3: AddrReg = AddrReg(3);
pub const A4: AddrReg = AddrReg(4);
pub const A5: AddrReg = AddrReg(5);
pub const A6: AddrReg = AddrReg(6);
pub const A7: AddrReg = AddrReg(7);

static ADDR_REGS: [AddrReg; 8] = [A0, A1, A2, A3, A4, A5, A6, A7];

impl Display for AddrReg {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%a{}", self.0)
    }
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
        s.fmt(f)
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

// Operands

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

    // Indirect
    AddrInd     (AddrReg),
    AddrIndInc  (AddrReg),
    AddrIndDec  (AddrReg),
    AddrDisp    (AddrReg, Const),
    AddrDispIdx (AddrReg, Const, Index),
    PcDisp      (         Const),
    PcDispIdx   (         Const, Index),

    // Special
    Regs(u8, u8), PC, SR, CCR, BC,
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
            PC          (..) => M_PC,
            SR          (..) => M_SR,
            CCR         (..) => M_CCR,
            BC          (..) => M_BC,
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
            Regs        (d, a)                => fmt_regs(d, a, f),
            AddrInd     (ref r)               => write!(f, "({})",  r),
            AddrIndInc  (ref r)               => write!(f, "({})+", r),
            AddrIndDec  (ref r)               => write!(f, "-({})", r),
            AddrDisp    (ref b, ref d)        => write!(f, "({},{})",       b, d),
            AddrDispIdx (ref b, ref d, ref i) => write!(f, "({},{},{}*{})", b, d, i, 1),
            PcDisp      (       ref d)        => write!(f, "(%pc,{})",       d),
            PcDispIdx   (       ref d, ref i) => write!(f, "(%pc,{},{}*{})", d, i, 1),
            PC                                => write!(f, "%pc"),
            SR                                => write!(f, "%sr"),
            CCR                               => write!(f, "%ccr"),
            BC                                => write!(f, "bc"),
        }
    }
}

fn fmt_regs(data: u8, addr: u8, f: &mut Formatter) -> fmt::Result {
    let join =
    try!(fmt_list(data, &DATA_REGS, false, f));
    try!(fmt_list(addr, &ADDR_REGS, join,  f));
    Ok(())
}

fn fmt_list<R>(bits: u8, regs: &[R; 8], mut join: bool, f: &mut Formatter)
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

// Code Generator

pub struct CodeGen<W: io::Write> {
    out: W
}

impl<W> CodeGen<W> where W: io::Write {
    pub fn new(out: W) -> Self {
        CodeGen { out: out }
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
            match (s, d) {
                (&Data(_), _) if d.is(M_Dst) => self.write_insn_2("add", s, d),
                (_, &Data(_)) if s.is(M_Src) => self.write_insn_2("add", s, d),
                _                           => panic!("X")
            }
        }
        dst
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
        let s = format!("{}", Regs(0xE3, 0x3E));

        assert_eq!(s, "%d0-%d1/%d5-%d7/%a1-%a5");
    }
}

