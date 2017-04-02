// ColdFire Opcodes
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

use self::Words::*;
use self::Arch::*;

// Tabular approach, like that used in GNU binutils.

#[derive(Clone, Copy, Debug)]
pub struct Opcode {
    pub name: &'static str,
    pub len:  usize,
    pub bits: Words,
    pub mask: Words,
    pub args: &'static [Arg],
    pub arch: Arch,
}

#[derive(Clone, Copy, Debug)]
pub enum Words {
    One(u16),
    Two(u16, u16),
}

pub type BitPos = u8;

#[derive(Clone, Copy, Debug)]
pub enum Arg {
    /// Addressing mode + register spec (6 bits).
    Modes(Modes, BitPos),

    /// Data register (3 bits)
    DataReg(BitPos),

    /// Address register (3 bits)
    AddrReg(BitPos),

    /// Data or address register (4 bits)
    NormalReg(BitPos),

    /// Condition code register (not stored)
    Ccr,

    /// Condition code register (not stored)
    Sr,

    /// Control register (12 bits)
    CtlReg(BitPos),

    /// Debug control register (5 bits)
    DbgReg(BitPos),

    /// List of data and address registers (extension word)
    RegList,

    /// Cache (2 bits)
    Cache,

    /// Quick immediate (8 bits, signed)
    Quick(BitPos),

    /// Shift immediate (3 bits, 0 => 8)
    Shift(BitPos),
}

#[derive(Clone, Copy, Debug)]
pub struct Modes(u16);

pub const D:    Modes = Modes(1 <<  0); // data register direct
pub const A:    Modes = Modes(1 <<  1); // address register direct
pub const AI:   Modes = Modes(1 <<  2); // address register indirect
pub const AIPI: Modes = Modes(1 <<  3); // address register indirect, auto-increment
pub const AIPD: Modes = Modes(1 <<  4); // address register indirect, auto-decrement
pub const AID:  Modes = Modes(1 <<  5); // address register indirect, displacedj
pub const AIXD: Modes = Modes(1 <<  6); // address register indirect, indexed, displaced
pub const M16:  Modes = Modes(1 <<  7); // absolute signed 16-bit
pub const M32:  Modes = Modes(1 <<  8); // absolute unsigneed 32-bit
pub const I:    Modes = Modes(1 <<  9); // immediate
pub const PID:  Modes = Modes(1 << 10); // pc-relative, displaced
pub const PIXD: Modes = Modes(1 << 11); // pc-relative, indexed, displaced

pub const DST:  Modes = Modes(D.0 | A.0 | AI.0 | AIPI.0 | AIPD.0 | AIXD.0 | M16.0 | M32.0);
pub const SRC:  Modes = Modes(DST.0 | I.0 | PID.0 | PIXD.0);

#[derive(Clone, Copy, Debug)]
pub enum Arch {
    /// ColdFire ISA_A
    CfIsaA,
}

macro_rules! opcodes {
    {$( $name:ident . $suff:ident $bits:tt $mask:tt [ $( $( $arg:tt ):+ ),* ] $arch:expr ; )+} =>
    {
        static OPCODES: &'static [Opcode] = &[
            $(
                Opcode {
                    name: stringify!($name.$suff),
                    len:  0,
                    bits: words!($bits),
                    mask: words!($mask),
                    args: &[$( arg!($( $arg ):+) ),*],
                    arch: $arch,
                },
            )+
        ];
    };
}

macro_rules! words {
    { ($a:expr         ) } => { One($a    ) };
    { ($a:expr, $b:expr) } => { Two($a, $b) };
}

macro_rules! arg {
    { src  : $pos:expr } => { Arg::Modes(SRC, $pos) };
    { dst  : $pos:expr } => { Arg::Modes(DST, $pos) };
    { data : $pos:expr } => { Arg::DataReg($pos)    };
}

opcodes! {
//  MNEMONIC    WORDS             MASKS             OPERANDS          ARCHITECTURES
    move.b      (0x1000)          (0xF000)          [src:0, dst:6]    CfIsaA;
    move.w      (0x3000)          (0xF000)          [src:0, dst:6]    CfIsaA;
    move.l      (0x2000)          (0xF000)          [src:0, dst:6]    CfIsaA;

    muls.l      (0x4C00, 0x0400)  (0xFFC0, 0x8FFF)  [src:0, data:12]  CfIsaA;
    mulu.l      (0x4C00, 0x0000)  (0xFFC0, 0x8FFF)  [src:0, data:12]  CfIsaA;
}

