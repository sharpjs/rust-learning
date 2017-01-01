// MCF5307 Locations & Addressing Modes
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

#![allow(non_upper_case_globals)]
// ^ Because we like our M_* constants as they are.

use std::fmt::{self, Formatter};
use std::ops::BitOr;
use num::ToPrimitive;

use aex::ast::Expr;
use aex::util::DisplayWith;

use super::flavor::*;

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

impl<'a> DisplayWith<CfFlavor> for CfValue<'a> {
    fn fmt(&self, f: &mut Formatter, c: &CfFlavor) -> fmt::Result {
        match *self {
            CfValue::Imm         (ref e) => (c.base.fmt_imm)(e, f, c.base),
            CfValue::Abs16       (ref e) => (c.fmt_abs_16  )(f, c, e),
            CfValue::Abs32       (ref e) => (c.fmt_abs_32  )(f, c, e),
            CfValue::Data        (ref r) => r.fmt(f, c),
            CfValue::Addr        (ref r) => r.fmt(f, c),
            CfValue::AddrInd     (ref r) => r.fmt(f, c),
            CfValue::AddrIndDec  (ref r) => r.fmt(f, c),
            CfValue::AddrIndInc  (ref r) => r.fmt(f, c),
            CfValue::AddrDisp    (ref a) => a.fmt(f, c),
            CfValue::AddrDispIdx (ref a) => a.fmt(f, c),
            CfValue::PcDisp      (ref a) => a.fmt(f, c),
            CfValue::PcDispIdx   (ref a) => a.fmt(f, c),

            CfValue::Regs        (ref r) => r.fmt(f, c),
            CfValue::Ctrl        (ref r) => r.fmt(f, c),
            CfValue::Sr                  => (c.base.fmt_reg)(f, "sr" ),
            CfValue::Ccr                 => (c.base.fmt_reg)(f, "ccr"),
            CfValue::Bc                  => f.write_str("bc"),
        }
    }
}

// -----------------------------------------------------------------------------
// Data Registers

use self::DataReg::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum DataReg {
    D0, D1, D2, D3, D4, D5, D6, D7
}

static DATA_REGS: [DataReg; 8] = [
    D0, D1, D2, D3, D4, D5, D6, D7
];

static DATA_REG_NAMES: [&'static str; 8] = [
    "d0", "d1", "d2", "d3", "d4", "d5", "d6", "d7"
];

impl DataReg {
    #[inline]
    fn with_num(n: u8) -> Self {
        DATA_REGS[n as usize]
    }

    #[inline]
    fn num(self) -> u8 {
        self as u8
    }

    #[inline]
    fn name(self) -> &'static str {
        DATA_REG_NAMES[self as usize]
    }
}

impl DisplayWith<CfFlavor> for DataReg {
    #[inline]
    fn fmt(&self, f: &mut Formatter, c: &CfFlavor) -> fmt::Result {
        (c.base.fmt_reg)(f, self.name())
    }
}

// -----------------------------------------------------------------------------
// Address Registers

use self::AddrReg::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum AddrReg {
    A0, A1, A2, A3, A4, A5, A6, A7
}

static ADDR_REGS: [AddrReg; 8] = [
    A0, A1, A2, A3, A4, A5, A6, A7
];

static ADDR_REG_NAMES: [&'static str; 8] = [
    "a0", "a1", "a2", "a3", "a4", "a5", "fp", "sp"
];

impl AddrReg {
    #[inline]
    fn with_num(n: u8) -> Self {
        ADDR_REGS[n as usize]
    }

    #[inline]
    fn num(self) -> u8 {
        self as u8
    }

    #[inline]
    fn name(self) -> &'static str {
        ADDR_REG_NAMES[self as usize]
    }
}

impl DisplayWith<CfFlavor> for AddrReg {
    #[inline]
    fn fmt(&self, f: &mut Formatter, c: &CfFlavor) -> fmt::Result {
        (c.base.fmt_reg)(f, self.name())
    }
}

// -----------------------------------------------------------------------------
// Control Registers

#[derive(Clone, Copy, /*Hash,*/ PartialEq, Eq, Debug)]
pub enum CtrlReg {
    VBR, CACR, ACR0, ACR1, MBAR, RAMBAR
}

static CTRL_REG_NAMES: [&'static str; 6] = [
    "vbr", "cacr", "acr0", "acr1", "mbar", "rambar"
];

impl CtrlReg {
    #[inline]
    fn name(self) -> &'static str {
        CTRL_REG_NAMES[self as usize]
    }
}

impl DisplayWith<CfFlavor> for CtrlReg {
    #[inline]
    fn fmt(&self, f: &mut Formatter, c: &CfFlavor) -> fmt::Result {
        (c.base.fmt_reg)(f, self.name())
    }
}

// -------------------------------------------------------------------------
// Program Counter Register

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PcReg;

impl DisplayWith<CfFlavor> for PcReg {
    #[inline]
    fn fmt(&self, f: &mut Formatter, c: &CfFlavor) -> fmt::Result {
        (c.base.fmt_reg)(f, "pc")
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

impl DisplayWith<CfFlavor> for RegSet {
    #[inline]
    fn fmt(&self, f: &mut Formatter, c: &CfFlavor) -> fmt::Result {
        let join =
        try!((c.fmt_data_regs)((self.0 & 0xFF) as u8, &DATA_REGS, false, f, c));
        try!((c.fmt_addr_regs)((self.0 >>   8) as u8, &ADDR_REGS, join,  f, c));
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Base (for displaced and indexed addressing modes)

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Base {
    Addr (AddrReg),
    Pc
}

impl DisplayWith<CfFlavor> for Base {
    #[inline]
    fn fmt(&self, f: &mut Formatter, c: &CfFlavor) -> fmt::Result {
        match *self {
            Base::Addr(ref r) =>     r.fmt(f, c),
            Base::Pc          => PcReg.fmt(f, c),
        }
    }
}

// -----------------------------------------------------------------------------
// Index (for indexed addressing modes)

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Index {
    Data (DataReg),
    Addr (AddrReg),
}

impl DisplayWith<CfFlavor> for Index {
    #[inline]
    fn fmt(&self, f: &mut Formatter, c: &CfFlavor) -> fmt::Result {
        match *self {
            Index::Data(ref r) => r.fmt(f, c),
            Index::Addr(ref r) => r.fmt(f, c),
        }
    }
}

// -----------------------------------------------------------------------------
// Address Register Base + Displacement

#[derive(Clone, PartialEq, Eq, /*Hash,*/ Debug)]
pub struct AddrDisp<'a> {
    pub base: AddrReg,
    pub disp: Expr<'a>
}

impl<'a> DisplayWith<CfFlavor> for AddrDisp<'a> {
    #[inline]
    fn fmt(&self, f: &mut Formatter, c: &CfFlavor) -> fmt::Result {
        (c.fmt_addr_disp)(f, c, self)
    }
}

// -----------------------------------------------------------------------------
// Address Register Base + Displacement + Index

#[derive(Clone, PartialEq, Eq, /*Hash,*/ Debug)]
pub struct AddrDispIdx<'a> {
    pub base:  AddrReg,
    pub disp:  Expr<'a>,
    pub index: Index,
    pub scale: Expr<'a>
}

impl<'a> DisplayWith<CfFlavor> for AddrDispIdx<'a> {
    #[inline]
    fn fmt(&self, f: &mut Formatter, c: &CfFlavor) -> fmt::Result {
        (c.fmt_addr_disp_idx)(f, c, self)
    }
}

// -----------------------------------------------------------------------------
// Program Counter + Displacement

#[derive(Clone, PartialEq, Eq, /*Hash,*/ Debug)]
pub struct PcDisp<'a> {
    pub disp: Expr<'a>
}

impl<'a> DisplayWith<CfFlavor> for PcDisp<'a> {
    #[inline]
    fn fmt(&self, f: &mut Formatter, c: &CfFlavor) -> fmt::Result {
        (c.fmt_pc_disp)(f, c, self)
    }
}

// -----------------------------------------------------------------------------
// Program Counter + Displacement + Index

#[derive(Clone, PartialEq, Eq, /*Hash,*/ Debug)]
pub struct PcDispIdx<'a> {
    pub disp:  Expr<'a>,
    pub index: Index,
    pub scale: Expr<'a>
}

impl<'a> DisplayWith<CfFlavor> for PcDispIdx<'a> {
    #[inline]
    fn fmt(&self, f: &mut Formatter, c: &CfFlavor) -> fmt::Result {
        (c.fmt_pc_disp_idx)(f, c, self)
    }
}

// -----------------------------------------------------------------------------
// Tests

//#[cfg(test)]
//mod tests {
//    //use super::*;
//    use super::DataReg::*;
//    use super::AddrReg::*;
//
//    #[test]
//    fn fmt_regs() {
//        let s = format!("{}", D0 | D3 | D6 | D7 | A1 | A2 | A3);
//        assert_eq!(s, "%d0/%d3/%d6-%d7/%a1-%a3");
//    }
//}

