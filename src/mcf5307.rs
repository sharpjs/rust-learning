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
use num::{BigInt, ToPrimitive};

use ast::Expr;
use types::*;
use util::*;
use util::shared::*;

// -----------------------------------------------------------------------------
// Addressing Modes

// NOTE: Required because stable Rust does not provide an API to read enum
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
// Locations - places where data can be read or written

pub trait Loc : LocEq + Display + Debug {
    fn mode(&self) -> Mode;
    fn is_q(&self) -> bool { false }
}

impl<'a> Loc + 'a {
    #[inline(always)]
    fn is(&self, modes: Mode) -> bool {
        self.mode() & modes != 0
    }
}

impl<'a, T: 'a + Loc> From<T> for Shared<'a, Loc> {
    #[inline(always)]
    fn from(t: T) -> Self { Shared::from(Rc::new(t) as Rc<Loc>) }
}

derive_dynamic_eq!(Loc : LocEq);

// Constant Expressions (defined in ast.rs)

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Expr::Ident      (ref s)              => f.write_str(&*s),
            Expr::Str        (ref s)              => fmt_str(&*s, f),
            Expr::Int        (ref i)              => fmt_int(  i, f),
            Expr::Negate     (ref e, None)        => write!(f, "-{}", e),
            Expr::Complement (ref e, None)        => write!(f, "~{}", e),
            Expr::Multiply   (ref l, ref r, None) => write!(f, "({} * {})",  l, r),
            Expr::Divide     (ref l, ref r, None) => write!(f, "({} / {})",  l, r),
            Expr::Modulo     (ref l, ref r, None) => write!(f, "({} % {})",  l, r),
            Expr::Add        (ref l, ref r, None) => write!(f, "({} + {})",  l, r),
            Expr::Subtract   (ref l, ref r, None) => write!(f, "({} - {})",  l, r),
            Expr::ShiftL     (ref l, ref r, None) => write!(f, "({} << {})", l, r),
            Expr::ShiftR     (ref l, ref r, None) => write!(f, "({} >> {})", l, r),
            Expr::BitAnd     (ref l, ref r, None) => write!(f, "({} & {})",  l, r),
            Expr::BitXor     (ref l, ref r, None) => write!(f, "({} ^ {})",  l, r),
            Expr::BitOr      (ref l, ref r, None) => write!(f, "({} | {})",  l, r),
            _                                     => f.write_str("**ERROR**")
        }
    }
}

fn fmt_str(s: &str, f: &mut Formatter) -> fmt::Result {
    try!(f.write_char('"'));
    for c in s.chars() {
        match c {
            '\x08'          => try!(f.write_str("\\b")),
            '\x09'          => try!(f.write_str("\\t")),
            '\x0A'          => try!(f.write_str("\\n")),
            '\x0C'          => try!(f.write_str("\\f")),
            '\x0D'          => try!(f.write_str("\\r")),
            '\"'            => try!(f.write_str("\\\"")),
            '\\'            => try!(f.write_str("\\\\")),
            '\x20'...'\x7E' => try!(f.write_char(c)),
            _               => try!(fmt_esc_utf8(c, f))
        }
    }
    try!(f.write_char('"'));
    Ok(())
}

fn fmt_esc_utf8(c: char, f: &mut Formatter) -> fmt::Result {
    use std::io::{Cursor, Write};
    let mut buf = [0u8; 4];
    let len = {
        let mut cur = Cursor::new(&mut buf[..]);
        write!(cur, "{}", c).unwrap();
        cur.position() as usize
    };
    for b in &buf[0..len] {
        try!(write!(f, "\\{:03o}", b));
    }
    Ok(())
}

fn fmt_int(i: &BigInt, f: &mut Formatter) -> fmt::Result {
    match i.to_u64() {
        Some(n) if n > 9 => write!(f, "{:#X}", n),
        _                => write!(f, "{}",    i),
    }
}

// Immediate

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Imm (Expr);

impl Loc for Imm {
    fn mode(&self) -> Mode { M_Imm }
    fn is_q(&self) -> bool {
        if let Expr::Int(ref i) = self.0 {
            if let Some(n) = i.to_u8() {
                return 1 <= n && n <= 8
            }
        }
        false
    }
}

impl Display for Imm {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "#{}", &self.0)
    }
}

// Distance - for absolute addressing and branches

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Distance { Near, Far }
use self::Distance::*;

// Absolute

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Abs {
    addr: Expr,
    dist: Distance
}

impl Loc for Abs {
    fn mode(&self) -> Mode {
        match self.dist { Near => M_Abs16, Far => M_Abs32 }
    }
}

impl Display for Abs {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let s = match self.dist { Near => "w", Far => "l" };
        write!(f, "{}:{}", &self.addr, s)
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
    fn mode(&self) -> Mode { M_Data }
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
    fn mode(&self) -> Mode { M_Addr }
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
    fn mode(&self) -> Mode { M_Regs }
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
    fn mode(&self) -> Mode { M_Ctrl }
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
    fn mode(&self) -> Mode { M_SR }
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
    fn mode(&self) -> Mode { M_CCR }
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
    fn mode(&self) -> Mode { M_BC }
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
    fn mode(&self) -> Mode { M_AddrInd }
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
    fn mode(&self) -> Mode { M_AddrIndDec }
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
    fn mode(&self) -> Mode { M_AddrIndInc }
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
    disp: Expr
}

impl Loc for AddrDisp {
    fn mode(&self) -> Mode { M_AddrDisp }
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
    disp:  Expr,
    index: Index,
    scale: Expr
}

impl Loc for AddrDispIdx {
    fn mode(&self) -> Mode { M_AddrDispIdx }
}

impl Display for AddrDispIdx {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {}, {}*{})", &self.base, &self.disp, &self.index, &self.scale)
    }
}

// Program Counter + Displacement

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PcDisp {
    disp: Expr
}

impl Loc for PcDisp {
    fn mode(&self) -> Mode { M_PcDisp }
}

impl Display for PcDisp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "(%pc, {})", &self.disp)
    }
}

// Program Counter + Displacement + Index

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PcDispIdx {
    disp:  Expr,
    index: Index,
    scale: Expr
}

impl Loc for PcDispIdx {
    fn mode(&self) -> Mode { M_PcDispIdx }
}

impl Display for PcDispIdx {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "(%pc, {}, {}*{})", &self.disp, &self.index, &self.scale)
    }
}

// -----------------------------------------------------------------------------
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

impl Display for Operand {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&*self.loc, f)
    }
}

// -----------------------------------------------------------------------------
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
            Expr::Add(ref src, ref dst, ref sel) => {
                let src = self.visit_expr(src);
                let dst = self.visit_expr(dst);
                // TODO: interpret sel
                self.add_data(src, dst)
            },
            Expr::Int(_) => {
                Operand::new(Imm(e.clone()), INT, Pos::bof())
            }
            _ => {
                panic!("not supported yet");
            }
        }
    }

    pub fn add(&mut self, expr: &Expr, src: Operand, dst: Operand, sel: &str) -> Operand {
        let modes = (src.loc.mode(), dst.loc.mode(), sel);
        match modes {
            (M_Imm,  M_Imm,  _  )                      => self.add_const(expr, src, dst),
            (M_Data, _,      "g") if dst.loc.is(M_Dst) => self.add_data(src, dst),
            (_,      M_Data, "g") if src.loc.is(M_Src) => self.add_data(src, dst),
            // ...others...
            (M_Data, _,      _  ) if dst.loc.is(M_Dst) => self.add_data(src, dst),
            (_,      M_Data, _  ) if src.loc.is(M_Src) => self.add_data(src, dst),
            _                                          => dst
        }
    }

    pub fn add_data(&mut self, src: Operand, dst: Operand) -> Operand {
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

    fn add_const(&mut self, expr: &Expr, src: Operand, dst: Operand) -> Operand {
        let a   = src.loc.downcast_ref::<Imm>().unwrap();
        let b   = dst.loc.downcast_ref::<Imm>().unwrap();
        let loc = match (&a.0, &b.0) {
            (&Expr::Int(ref a), &Expr::Int(ref b)) => Imm(Expr::Int(a + b)),
            _                                      => Imm(expr.clone())
        };
        Operand::new(loc, INT, src.pos)
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
    use num::bigint::ToBigInt;

    use super::*;
    use super::DataReg::*;
    use super::AddrReg::*;
    use ast::Expr;
    use types::*;
    use util::*;

    #[test]
    fn foo() {
        let n   = 4u8.to_bigint().unwrap();
        let src = Operand::new(Imm(Expr::Int(n)), U8, Pos::bof());
        let dst = Operand::new(D3,                U8, Pos::bof());

        let mut gen = CodeGen::new(io::stdout());
        let res = gen.add_data(src, dst.clone());

        assert_eq!(dst, res);
    }

    #[test]
    fn fmt_regs() {
        let s = format!("{}", D0 | D3 | D6 | D7 | A1 | A2 | A3);
        assert_eq!(s, "%d0/%d3/%d6-%d7/%a1-%a3");
    }
}

