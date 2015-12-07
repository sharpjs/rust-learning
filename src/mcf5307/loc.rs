// MCF5307 Locations & Addressing Modes
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

use num::ToPrimitive;
use std::fmt::{self, Display, Formatter, Write};
use std::ops::BitOr;

use ast::Expr;

use self::AddrReg::*;
use self::DataReg::*;

// -----------------------------------------------------------------------------
// Addressing Modes

// NOTE: Required because Rust does not provide a stable API to read enum
// discriminator values.

type Mode = u32;
const M_Imm:         Mode = 1 <<  0;
const M_Abs16:       Mode = 1 <<  1;
const M_Abs32:       Mode = 1 <<  2;
const M_Data:        Mode = 1 <<  3;
const M_Addr:        Mode = 1 <<  4;
const M_Ctrl:        Mode = 1 <<  5;
const M_Regs:        Mode = 1 <<  6;
const M_AddrInd:     Mode = 1 <<  7;
const M_AddrIndInc:  Mode = 1 <<  8;
const M_AddrIndDec:  Mode = 1 <<  9;
const M_AddrDisp:    Mode = 1 << 10;
const M_AddrDispIdx: Mode = 1 << 11;
const M_PcDisp:      Mode = 1 << 12;
const M_PcDispIdx:   Mode = 1 << 13;
const M_PC:          Mode = 1 << 14;
const M_SR:          Mode = 1 << 15;
const M_CCR:         Mode = 1 << 16;
const M_BC:          Mode = 1 << 17;

const M_Reg: Mode
    = M_Data | M_Addr;

const M_Dst: Mode
    = M_Reg | M_AddrInd | M_AddrIndInc | M_AddrIndDec | M_AddrDisp | M_AddrDispIdx;

const M_Src: Mode
    = M_Dst | M_Imm | M_PcDisp | M_PcDispIdx;

// -----------------------------------------------------------------------------
// Loc - a location where data can be read or written

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum Loc<'a> {
    Imm         (Expr<'a>),
    Abs16       (Expr<'a>),
    Abs32       (Expr<'a>),
    Data        (DataReg),
    Addr        (AddrReg),
    AddrInd     (AddrReg),
    AddrIndDec  (AddrReg),
    AddrIndInc  (AddrReg),
    AddrDisp    (AddrDisp   <'a>),
    AddrDispIdx (AddrDispIdx<'a>),
    PcDisp      (PcDisp     <'a>),
    PcDispIdx   (PcDispIdx  <'a>),

    Regs        (RegSet),
    Ctrl        (CtrlReg),
    Sr,
    Ccr,
    Bc,
}

impl<'a> Loc<'a> {
    fn mode(&self) -> Mode {
        match *self {
            Loc::Imm         (..) => M_Imm,
            Loc::Abs16       (..) => M_Abs16,
            Loc::Abs32       (..) => M_Abs32,
            Loc::Data        (..) => M_Data,
            Loc::Addr        (..) => M_Addr,
            Loc::AddrInd     (..) => M_AddrInd,
            Loc::AddrIndDec  (..) => M_AddrIndDec,
            Loc::AddrIndInc  (..) => M_AddrIndInc,
            Loc::AddrDisp    (..) => M_AddrDisp,
            Loc::AddrDispIdx (..) => M_AddrDispIdx,
            Loc::PcDisp      (..) => M_PcDisp,
            Loc::PcDispIdx   (..) => M_PcDispIdx,

            Loc::Regs        (..) => M_Regs,
            Loc::Ctrl        (..) => M_Ctrl,
            Loc::Sr               => M_SR,
            Loc::Ccr              => M_CCR,
            Loc::Bc               => M_BC,
        }
    }

    fn is(&self, modes: Mode) -> bool {
        self.mode() & modes != 0
    }

    fn is_q(&self) -> bool {
        match *self {
            Loc::Imm(Expr::Int(ref n)) => {
                match n.to_u8() {
                    Some(n) => 1 <= n && n <= 8,
                    None    => false
                }
            },
            Loc::Imm(_) => true, // let assembler figure it out
            _ => false
        }
    }
}

impl<'a> Display for Loc<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Loc::Imm         (ref e) => write!(f, "#{}",  e),
            Loc::Abs16       (ref e) => write!(f, "{}:w", e),
            Loc::Abs32       (ref e) => write!(f, "{}:l", e),
            Loc::Data        (ref r) => r.fmt(f),
            Loc::Addr        (ref r) => r.fmt(f),
            Loc::AddrInd     (ref r) => write!(f, "({})", r),
            Loc::AddrIndDec  (ref r) => write!(f, "-({})", r),
            Loc::AddrIndInc  (ref r) => write!(f, "({})+", r),
            Loc::AddrDisp    (ref a) => a.fmt(f),
            Loc::AddrDispIdx (ref a) => a.fmt(f),
            Loc::PcDisp      (ref a) => a.fmt(f),
            Loc::PcDispIdx   (ref a) => a.fmt(f),

            Loc::Regs        (ref r) => r.fmt(f),
            Loc::Ctrl        (ref r) => r.fmt(f),
            Loc::Sr               => f.write_str("%sr"),
            Loc::Ccr              => f.write_str("%ccr"),
            Loc::Bc               => f.write_str("bc"),
        }
    }
}

// Data Registers

#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(u8)]
pub enum DataReg { D0, D1, D2, D3, D4, D5, D6, D7 }

static DATA_REGS: [DataReg; 8] = [D0, D1, D2, D3, D4, D5, D6, D7];

impl DataReg {
    fn with_num(n: u8) -> Self {
        DATA_REGS[n as usize]
    }

    fn num(self) -> u8 {
        self as u8
    }
}

impl Display for DataReg {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%d{}", self.num())
    }
}

// Address Registers

#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(u8)]
pub enum AddrReg { A0, A1, A2, A3, A4, A5, A6, A7 }

static ADDR_REGS: [AddrReg; 8] = [A0, A1, A2, A3, A4, A5, A6, A7];

impl AddrReg {
    fn with_num(n: u8) -> Self {
        ADDR_REGS[n as usize]
    }

    fn num(self) -> u8 {
        self as u8
    }
}

impl Display for AddrReg {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%a{}", self.num())
    }
}

// Control Registers

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
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

// Register Set

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
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

// Index (for indexed addressing modes)

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
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

// Address Register Base + Displacement

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct AddrDisp<'a> {
    base: AddrReg,
    disp: Expr<'a>
}

impl<'a> Display for AddrDisp<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {})", &self.base, &self.disp)
    }
}

// Address Register Base + Displacement + Index

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct AddrDispIdx<'a> {
    base:  AddrReg,
    disp:  Expr<'a>,
    index: Index,
    scale: Expr<'a>
}

impl<'a> Display for AddrDispIdx<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {}, {}*{})", &self.base, &self.disp, &self.index, &self.scale)
    }
}

// Program Counter + Displacement

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct PcDisp<'a> {
    disp: Expr<'a>
}

impl<'a> Display for PcDisp<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "(%pc, {})", &self.disp)
    }
}

// Program Counter + Displacement + Index

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct PcDispIdx<'a> {
    disp:  Expr<'a>,
    index: Index,
    scale: Expr<'a>
}

impl<'a> Display for PcDispIdx<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "(%pc, {}, {}*{})", &self.disp, &self.index, &self.scale)
    }
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    //use super::*;
    use super::DataReg::*;
    use super::AddrReg::*;

    #[test]
    fn fmt_regs() {
        let s = format!("{}", D0 | D3 | D6 | D7 | A1 | A2 | A3);
        assert_eq!(s, "%d0/%d3/%d6-%d7/%a1-%a3");
    }
}

