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

use types::*;
use util::*;

// Constants

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Const {
    Name  (String),
    Const (u64), // TODO: make this work with types
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

// Control Registers

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum CtrlReg { VBR, CACR, ACR0, ACR1, MBAR, RAMBAR }

// Index Registers

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum Index {
    Data (DataReg),
    Addr (AddrReg),
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
pub struct Operand (Pos, &'static Type, Mode);

impl Mode {
    fn uses(&self) -> Uses {
        match *self {
            Imm         (..) => U_SRC         ,
            Abs16       (..) => U_SRC | U_DST ,    
            Abs32       (..) => U_SRC | U_DST ,
            Data        (..) => U_SRC | U_DST ,
            Addr        (..) => U_SRC | U_DST ,
            Ctrl        (..) =>         U_DST ,
            AddrInd     (..) => U_SRC | U_DST ,
            AddrIndInc  (..) => U_SRC | U_DST ,
            AddrIndDec  (..) => U_SRC | U_DST ,
            AddrDisp    (..) => U_SRC | U_DST ,
            AddrDispIdx (..) => U_SRC | U_DST ,
            PcDisp      (..) => U_SRC         ,
            PcDispIdx   (..) => U_SRC         ,
            _                => U_NONE
        }
    }

    fn is_q(&self) -> bool {
        match *self {
            Imm(Const::Const(i)) => 1 <= i && i <= 8,
            _ => false
        }
    }

    fn is_src(&self) -> bool { self.uses() & U_SRC != 0 }
    fn is_dst(&self) -> bool { self.uses() & U_DST != 0 }
}

pub fn add_g<S, D>(src: S, dst: D) -> D
    where S: Borrow<Operand>,
          D: Borrow<Operand>
{
    {
        let &Operand(_, _, ref sm) = src.borrow();
        let &Operand(_, _, ref dm) = dst.borrow();

        match (sm, dm) {
            (&Data(_), _) if dm.is_dst() => write_insn_2(sm, dm),
            (_, &Data(_)) if sm.is_src() => write_insn_2(sm, dm),
            _            => {}
        }
    }
    dst
}

fn write_insn_2(src: &Mode, dst: &Mode) {
    // TODO
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::*;
    use util::*;

    #[test]
    fn foo() {
        let src = Operand(Pos::bof(), &U8, Mode::Imm(Const::Const(4)));
        let dst = Operand(Pos::bof(), &U8, Mode::Data(D0));
        let res = add_g(&src, &dst);
        assert_eq!(&dst, res);
    }
}

