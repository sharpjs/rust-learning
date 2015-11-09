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

use std::borrow::Borrow;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Write};

use types::*;
use util::*;

pub trait Modey : Display {
    fn uses   (&self) -> Uses;
    fn is_q   (&self) -> bool { false }
    fn is_src (&self) -> bool { self.uses() & U_SRC != 0 }
    fn is_dst (&self) -> bool { self.uses() & U_DST != 0 }
}

impl Modey for DataReg {
    fn uses(&self) -> Uses { U_SRC | U_DST }
}

// Immediate

impl Display for Const {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            &Const::Sym(ref s) => write!(f, "?{}",   s),
            &Const::Num(ref v) => write!(f, "{:#X}", v),
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
enum CtrlReg { VBR, CACR, ACR0, ACR1, MBAR, RAMBAR }

impl Display for CtrlReg {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let s = match self {
            &CtrlReg::VBR    => "vbr",
            &CtrlReg::CACR   => "cacr",
            &CtrlReg::ACR0   => "acr0",
            &CtrlReg::ACR1   => "acr1",
            &CtrlReg::MBAR   => "mbar",
            &CtrlReg::RAMBAR => "rambar",
        };
        write!(f, "%{}", s)
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
        match self {
            &Index::Data(ref r) => r.fmt(f),
            &Index::Addr(ref r) => r.fmt(f),
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
    Regs        (Vec<DataReg>, Vec<AddrReg>),

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

type Uses = u16;
const U_NONE: Uses = 0;
const U_SRC:  Uses = 1 << 0;
const U_DST:  Uses = 1 << 1;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Operand<'a> {
    pos:    Pos,
    ty:     &'a Type,
    mode:   Mode
}

impl Mode {
    fn uses(&self) -> Uses {
        match self {
            &Imm         (..) => U_SRC         ,
            &PcDisp      (..) => U_SRC         ,
            &PcDispIdx   (..) => U_SRC         ,
            &Abs16       (..) => U_SRC | U_DST ,    
            &Abs32       (..) => U_SRC | U_DST ,
            &Data        (..) => U_SRC | U_DST ,
            &Addr        (..) => U_SRC | U_DST ,
            &AddrInd     (..) => U_SRC | U_DST ,
            &AddrIndInc  (..) => U_SRC | U_DST ,
            &AddrIndDec  (..) => U_SRC | U_DST ,
            &AddrDisp    (..) => U_SRC | U_DST ,
            &AddrDispIdx (..) => U_SRC | U_DST ,
            &Ctrl        (..) =>         U_DST ,
            _                 => U_NONE
        }
    }

    fn is_q(&self) -> bool {
        match self {
            &Imm(Const::Num(i)) => 1 <= i && i <= 8,
            _ => false
        }
    }

    fn is_src(&self) -> bool { self.uses() & U_SRC != 0 }
    fn is_dst(&self) -> bool { self.uses() & U_DST != 0 }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Imm         (ref v)               => write!(f, "#{}",  v),
            &Abs16       (ref v)               => write!(f, "{}:w", v),
            &Abs32       (ref v)               => write!(f, "{}:l", v),
            &Data        (ref r)               => r.fmt(f),
            &Addr        (ref r)               => r.fmt(f),
            &Ctrl        (ref r)               => r.fmt(f),
            &Regs        (..)                  => write!(f, "?"),
            &AddrInd     (ref r)               => write!(f, "({})",             r),
            &AddrIndInc  (ref r)               => write!(f, "({})+",            r),
            &AddrIndDec  (ref r)               => write!(f, "-({})",            r),
            &AddrDisp    (ref b, ref d)        => write!(f, "({}, {})",         b, d),
            &AddrDispIdx (ref b, ref d, ref i) => write!(f, "({}, {}, {}*{})",  b, d, i, 1),
            &PcDisp      (       ref d)        => write!(f, "(%pc, {})",           d),
            &PcDispIdx   (       ref d, ref i) => write!(f, "(%pc, {}, {}*{})",    d, i, 1),
            &PC                                => write!(f, "%pc"),
            &SR                                => write!(f, "%sr"),
            &CCR                               => write!(f, "%ccr"),
            &BC                                => write!(f, "bc"),
        }
    }
}

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
                (&Data(_), _) if d.is_dst() => self.write_insn_2("add", s, d),
                (_, &Data(_)) if s.is_src() => self.write_insn_2("add", s, d),
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
}

