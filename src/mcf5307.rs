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

use std::fmt::{self, Display, Debug, Formatter, Write};
use std::ops::BitOr;
use std::io;
use std::rc::Rc;

use ast::Expr;
use types::*;
use util::*;
use util::shared::*;

// Locations - place where data can be read or written

pub trait Loc : LocEq + Display + Debug {
    fn mode(&self) -> ModeId;
    fn is_q(&self) -> bool { false }
}

impl<'a> Loc + 'a {
    #[inline(always)]
    fn is(&self, modes: ModeId) -> bool {
        self.mode() & modes != 0
    }
}

impl<'a, T: 'a + Loc> From<T> for Shared<'a, Loc> {
    #[inline(always)]
    fn from(t: T) -> Self { Shared::from(Rc::new(t) as Rc<Loc>) }
}

derive_dynamic_eq!(Loc : LocEq);

// Constants (defined in types.rs)

impl Display for Const {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Const::Sym(ref s)           => f.write_str(s),
            Const::Num(    v) if v < 10 => write!(f, "{}",    v),
            Const::Num(    v)           => write!(f, "{:#X}", v),
        }
    }
}

// Immediate

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Imm (Const);

impl Loc for Imm {
    fn mode(&self) -> ModeId { M_Imm }
    fn is_q(&self) -> bool {
        match self.0 {
            Const::Num(i) => 1 <= i && i <= 8,
            _             => false
        }
    }
}

impl Display for Imm {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "#{}", &self.0)
    }
}

// Absolute

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Abs {
    addr: Const,
    // width?
}

impl Loc for Abs {
    fn mode(&self) -> ModeId { M_Abs32 }
}

impl Display for Abs {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:l", &self.addr)
    }
}

// Data Registers

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(u8)]
pub enum DataReg { D0, D1, D2, D3, D4, D5, D6, D7 }
use self::DataReg::*;

static DATA_REGS: [DataReg; 8] = [D0, D1, D2, D3, D4, D5, D6, D7];

impl DataReg {
    fn with_num(n: u8) -> Self {
        DATA_REGS[n as usize]
    }
    fn num(self) -> u8 {
        self as u8
    }
}

impl Loc for DataReg {
    fn mode(&self) -> ModeId { M_Data }
}

impl Display for DataReg {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%d{}", self.num())
    }
}

// Address Registers

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(u8)]
pub enum AddrReg { A0, A1, A2, A3, A4, A5, A6, A7 }
use self::AddrReg::*;

static ADDR_REGS: [AddrReg; 8] = [A0, A1, A2, A3, A4, A5, A6, A7];

impl AddrReg {
    fn with_num(n: u8) -> Self {
        ADDR_REGS[n as usize]
    }
    fn num(self) -> u8 {
        self as u8
    }
}

impl Loc for AddrReg {
    fn mode(&self) -> ModeId { M_Addr }
}

impl Display for AddrReg {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%a{}", self.num())
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

impl Loc for RegSet {
    fn mode(&self) -> ModeId { M_Regs }
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

impl Loc for CtrlReg {
    fn mode(&self) -> ModeId { M_Ctrl }
}

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

// Status Register

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct SR;

impl Loc for SR {
    fn mode(&self) -> ModeId { M_SR }
}

impl Display for SR {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("%sr")
    }
}

// Condition Code Register

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct CCR;

impl Loc for CCR {
    fn mode(&self) -> ModeId { M_CCR }
}

impl Display for CCR {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("%ccr")
    }
}

// Both Caches Specifier

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct BC;

impl Loc for BC {
    fn mode(&self) -> ModeId { M_BC }
}

impl Display for BC {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("bc")
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
            Index::Data(ref r) => <Display>::fmt(r, f),
            Index::Addr(ref r) => <Display>::fmt(r, f),
        }
    }
}

// Address Register Indirect

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct AddrInd {
    reg: AddrReg
}

impl Loc for AddrInd {
    fn mode(&self) -> ModeId { M_AddrInd }
}

impl Display for AddrInd {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({})", &self.reg)
    }
}

// Address Register Indirect With Pre-Decrement

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct AddrIndDec {
    reg: AddrReg
}

impl Loc for AddrIndDec {
    fn mode(&self) -> ModeId { M_AddrIndDec }
}

impl Display for AddrIndDec {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "-({})", &self.reg)
    }
}

// Address Register Indirect With Post-Increment

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct AddrIndInc {
    reg: AddrReg
}

impl Loc for AddrIndInc {
    fn mode(&self) -> ModeId { M_AddrIndInc }
}

impl Display for AddrIndInc {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({})+", &self.reg)
    }
}

// Address Register Base + Displacement

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct AddrDisp {
    base: AddrReg,
    disp: Const
}

impl Loc for AddrDisp {
    fn mode(&self) -> ModeId { M_AddrDisp }
}

impl Display for AddrDisp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {})", &self.base, &self.disp)
    }
}

// Address Register Base + Displacement + Index

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct AddrDispIdx {
    base:  AddrReg,
    disp:  Const,
    index: Index,
    scale: Const
}

impl Loc for AddrDispIdx {
    fn mode(&self) -> ModeId { M_AddrDispIdx }
}

impl Display for AddrDispIdx {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {}, {}*{})", &self.base, &self.disp, &self.index, &self.scale)
    }
}

// Program Counter + Displacement

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PcDisp {
    disp: Const
}

impl Loc for PcDisp {
    fn mode(&self) -> ModeId { M_PcDisp }
}

impl Display for PcDisp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "(%pc, {})", &self.disp)
    }
}

// Program Counter + Displacement + Index

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PcDispIdx {
    disp:  Const,
    index: Index,
    scale: Const
}

impl Loc for PcDispIdx {
    fn mode(&self) -> ModeId { M_PcDispIdx }
}

impl Display for PcDispIdx {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "(%pc, {}, {}*{})", &self.disp, &self.index, &self.scale)
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

// Operand = a machine location with its analyzed type and source position

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Operand {
    pub loc: Shared<'static, Loc>,
    pub ty:  Shared<'static, Type>,
    pub pos: Pos,
}

impl Operand {
    pub fn new<L, T>(loc: L, ty: T, pos: Pos) -> Self
        where L: Into<Shared<'static, Loc>>,
              T: Into<Shared<'static, Type>>
    {
        Operand { loc: loc.into(), ty: ty.into(), pos: pos }
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

    pub fn visit_expr(&mut self, e: &Expr) -> Operand {
        match *e {
            Expr::Add(ref src, ref dst, sel) => {
                let src = self.visit_expr(src);
                let dst = self.visit_expr(dst);
                // TODO: interpret sel
                self.add_g(src, dst)
            },
            Expr::Int(n) => {
                Operand::new(Imm(Const::Num(n)), INT, Pos::bof())
            }
            _ => {
                panic!("not supported yet");
            }
        }
    }

    pub fn add_g(&mut self, src: Operand, dst: Operand) -> Operand
    {
        require_types_equal(&src, &dst);
        {
            let s = &*src.loc;
            let d = &*dst.loc;
            match (s.mode(), d.mode()) {
                (M_Data, _) if d.is(M_Dst) => self.write_insn_2("add", s, d),
                (_, M_Data) if s.is(M_Src) => self.write_insn_2("add", s, d),
                _                          => panic!("X")
            }
        }
        dst
    }

    #[allow(unused_must_use)]
    fn add_const(&mut self, src: &Loc, dst: &Loc) {
        let src = src.downcast_ref::<Imm>().unwrap();
        let dst = dst.downcast_ref::<Imm>().unwrap();
        match (&src.0, &dst.0) {
            (&Const::Num(a), &Const::Num(b)) => {
                write!(self.out, "{}", a + b);
            },
            (a, b) => {
                write!(self.out, "({} + {})", a, b);
            }
        }
    }

    fn write_insn_2(&mut self, op: &str, src: &Loc, dst: &Loc) {
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
    use std::io;

    use super::*;
    use super::DataReg::*;
    use super::AddrReg::*;
    use types::*;
    use util::*;

    #[test]
    fn foo() {
        let src = Operand::new(Imm(Const::Num(4)), U8, Pos::bof());
        let dst = Operand::new(D0,                 U8, Pos::bof());

        let mut gen = CodeGen::new(io::stdout());
        let res = gen.add_g(src, dst.clone());

        assert_eq!(dst, res);
    }

    #[test]
    fn fmt_regs() {
        let s = format!("{}", D0 | D3 | D6 | D7 | A1 | A2 | A3);
        assert_eq!(s, "%d0/%d3/%d6-%d7/%a1-%a3");
    }
}

