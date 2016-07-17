// MCF5307 Locations & Addressing Modes
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

#![allow(non_upper_case_globals)]
// ^ Because we like our M_* constants as they are.

use std::fmt::{self, Display, Formatter, Write};
use std::ops::BitOr;
use num::ToPrimitive;

use aex::ast::Expr;

use self::AddrReg::*;
use self::DataReg::*;

// -----------------------------------------------------------------------------
// Addressing Modes

// NOTE: Required because Rust does not provide a stable API to read enum
// discriminator values.

pub type Mode = u32;
pub const M_Imm:         Mode = 1 <<  0;
pub const M_Abs16:       Mode = 1 <<  1;
pub const M_Abs32:       Mode = 1 <<  2;
pub const M_Data:        Mode = 1 <<  3;
pub const M_Addr:        Mode = 1 <<  4;
pub const M_Ctrl:        Mode = 1 <<  5;
pub const M_Regs:        Mode = 1 <<  6;
pub const M_AddrInd:     Mode = 1 <<  7;
pub const M_AddrIndInc:  Mode = 1 <<  8;
pub const M_AddrIndDec:  Mode = 1 <<  9;
pub const M_AddrDisp:    Mode = 1 << 10;
pub const M_AddrDispIdx: Mode = 1 << 11;
pub const M_PcDisp:      Mode = 1 << 12;
pub const M_PcDispIdx:   Mode = 1 << 13;
pub const M_PC:          Mode = 1 << 14;
pub const M_SR:          Mode = 1 << 15;
pub const M_CCR:         Mode = 1 << 16;
pub const M_BC:          Mode = 1 << 17;

pub const M_Reg: Mode
    = M_Data | M_Addr;

pub const M_Dst: Mode
    = M_Reg | M_AddrInd | M_AddrIndInc | M_AddrIndDec | M_AddrDisp | M_AddrDispIdx;

pub const M_Src: Mode
    = M_Dst | M_Imm | M_PcDisp | M_PcDispIdx;

#[inline(always)]
pub fn mode_any(mode: Mode, modes: Mode) -> bool {
    mode & modes != 0
}

// -----------------------------------------------------------------------------
// Values

#[derive(Clone, /*Hash,*/ PartialEq, Eq, Debug)]
pub enum CfValue<'a> {
    // Normal
    Imm         (Expr<'a>),         // Immediate
    Abs16       (Expr<'a>),         // Absolute 16-bit value
    Abs32       (Expr<'a>),         // Absolute 32-bit value
    Data        (DataReg),          // Data register
    Addr        (AddrReg),          // Address register
    AddrInd     (AddrReg),          // Address register indirect
    AddrIndDec  (AddrReg),          // Address register indirect, pre-decrement
    AddrIndInc  (AddrReg),          // Address register indirect, post-increment
    AddrDisp    (AddrDisp   <'a>),  // Address register indirect, displaced
    AddrDispIdx (AddrDispIdx<'a>),  // Address register indirect, displaced, indexed
    PcDisp      (PcDisp     <'a>),  // PC-relative, displaced
    PcDispIdx   (PcDispIdx  <'a>),  // PC-relative, displaced, indexed

    // Special
    Regs        (RegSet),           // Multiple register (movem)
    Ctrl        (CtrlReg),          // Control register  (movec)
    Sr,                             // Status register
    Ccr,                            // Condition code register
    Bc,                             // Cache specifier (both i+d)
}

impl<'a> CfValue<'a> {
    pub fn mode(&self) -> Mode {
        match *self {
            CfValue::Imm         (..) => M_Imm,
            CfValue::Abs16       (..) => M_Abs16,
            CfValue::Abs32       (..) => M_Abs32,
            CfValue::Data        (..) => M_Data,
            CfValue::Addr        (..) => M_Addr,
            CfValue::AddrInd     (..) => M_AddrInd,
            CfValue::AddrIndDec  (..) => M_AddrIndDec,
            CfValue::AddrIndInc  (..) => M_AddrIndInc,
            CfValue::AddrDisp    (..) => M_AddrDisp,
            CfValue::AddrDispIdx (..) => M_AddrDispIdx,
            CfValue::PcDisp      (..) => M_PcDisp,
            CfValue::PcDispIdx   (..) => M_PcDispIdx,

            CfValue::Regs        (..) => M_Regs,
            CfValue::Ctrl        (..) => M_Ctrl,
            CfValue::Sr               => M_SR,
            CfValue::Ccr              => M_CCR,
            CfValue::Bc               => M_BC,
        }
    }

    pub fn is(&self, modes: Mode) -> bool {
        mode_any(self.mode(), modes)
    }

    pub fn is_q(&self) -> bool {
        match *self {
            CfValue::Imm(Expr::Int(ref n)) => {
                match n.val.to_u8() {
                    Some(n) => 1 <= n && n <= 8,
                    None    => false
                }
            },
            CfValue::Imm(_) => true, // let assembler figure it out
            _ => false
        }
    }

    pub fn as_expr(&self) -> &Expr<'a> {
        match *self {
            CfValue::Imm   (ref e) => e,
            CfValue::Abs16 (ref e) => e,
            CfValue::Abs32 (ref e) => e,
            _ => panic!("Cannot unwrap to expression.")
        }
    }
}

impl<'a> Display for CfValue<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            CfValue::Imm         (ref e) => write!(f, "#{}",    e),
            CfValue::Abs16       (ref e) => write!(f, "({}).w", e),
            CfValue::Abs32       (ref e) => write!(f, "({}).l", e),
            CfValue::Data        (ref r) => r.fmt(f),
            CfValue::Addr        (ref r) => r.fmt(f),
            CfValue::AddrInd     (ref r) => write!(f,  "({})",  r),
            CfValue::AddrIndDec  (ref r) => write!(f, "-({})",  r),
            CfValue::AddrIndInc  (ref r) => write!(f,  "({})+", r),
            CfValue::AddrDisp    (ref a) => a.fmt(f),
            CfValue::AddrDispIdx (ref a) => a.fmt(f),
            CfValue::PcDisp      (ref a) => a.fmt(f),
            CfValue::PcDispIdx   (ref a) => a.fmt(f),

            CfValue::Regs        (ref r) => r.fmt(f),
            CfValue::Ctrl        (ref r) => r.fmt(f),
            CfValue::Sr                  => f.write_str("%sr"),
            CfValue::Ccr                 => f.write_str("%ccr"),
            CfValue::Bc                  => f.write_str("bc"),
        }
    }
}

use aex::asm::AsmFlavor;
use aex::util::DisplayWith;

impl<'a> DisplayWith<AsmFlavor> for Expr<'a> {
    fn fmt(&self, f: &mut Formatter, a: &AsmFlavor) -> fmt::Result {
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Data Registers

#[derive(Clone, Copy, /*Hash,*/ PartialEq, Eq, PartialOrd, Ord, Debug)]
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

// -----------------------------------------------------------------------------
// Address Registers

#[derive(Clone, Copy, /*Hash,*/ PartialEq, Eq, PartialOrd, Ord, Debug)]
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

// -----------------------------------------------------------------------------
// Control Registers

#[derive(Clone, Copy, /*Hash,*/ PartialEq, Eq, Debug)]
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

// -----------------------------------------------------------------------------
// Register Set

#[derive(Clone, Copy, /*Hash,*/ PartialEq, Eq, Debug)]
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

// -----------------------------------------------------------------------------
// Index (for indexed addressing modes)

#[derive(Clone, Copy, /*Hash,*/ PartialEq, Eq, Debug)]
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

// -----------------------------------------------------------------------------
// Address Register Base + Displacement

#[derive(Clone, /*Hash,*/ PartialEq, Eq, Debug)]
pub struct AddrDisp<'a> {
    base: AddrReg,
    disp: Expr<'a>
}

impl<'a> Display for AddrDisp<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {})", &self.base, &self.disp)
    }
}

// -----------------------------------------------------------------------------
// Address Register Base + Displacement + Index

#[derive(Clone, /*Hash,*/ PartialEq, Eq, Debug)]
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

// -----------------------------------------------------------------------------
// Program Counter + Displacement

#[derive(Clone, /*Hash,*/ PartialEq, Eq, Debug)]
pub struct PcDisp<'a> {
    disp: Expr<'a>
}

impl<'a> Display for PcDisp<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "(%pc, {})", &self.disp)
    }
}

// -----------------------------------------------------------------------------
// Program Counter + Displacement + Index

#[derive(Clone, /*Hash,*/ PartialEq, Eq, Debug)]
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

