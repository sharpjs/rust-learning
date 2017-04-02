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

use self::Size::*;
use self::Words::*;

// -----------------------------------------------------------------------------

// Tabular approach, like that used in GNU binutils.

#[derive(Clone, Copy, Debug)]
pub struct Opcode {
    pub name: &'static str,
    pub size: Size,
    pub bits: Words,
    pub mask: Words,
    pub args: &'static [Arg],
    pub arch: Arch,
}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Size {
    Zero = 0,
    Byte = 1,
    Word = 2,
    Long = 4,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Words {
    One(u16),
    Two(u16, u16),
}

pub type BitPos = u8;

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

// -----------------------------------------------------------------------------

/// Addressing mode flags.
pub type Modes = u16;

pub const D:  Modes = 1 <<  0; // data reg direct
pub const A:  Modes = 1 <<  1; // addr reg direct
pub const AI: Modes = 1 <<  2; // addr reg indirect
pub const AP: Modes = 1 <<  3; // addr reg indirect, auto-increment
pub const AM: Modes = 1 <<  4; // addr reg indirect, auto-decrement
pub const AD: Modes = 1 <<  5; // addr reg indirect, displaced
pub const AX: Modes = 1 <<  6; // addr reg indirect, indexed, displaced
pub const MS: Modes = 1 <<  7; // absolute short
pub const ML: Modes = 1 <<  8; // absolute long
pub const I:  Modes = 1 <<  9; // immediate
pub const PD: Modes = 1 << 10; // pc-relative, displaced
pub const PX: Modes = 1 << 11; // pc-relative, indexed, displaced

pub const DST: Modes = D | A | AI | AP | AM | AD | AX | MS | ML;
pub const SRC: Modes = DST | I | PD | PX;

pub const DST_MEM: Modes = DST & !D & !A;

// -----------------------------------------------------------------------------

/// Architecture flags.
pub type Arch = u16;

pub const RELAX: Arch = 1 << 0; // Relaxations enabled
pub const CF_A:  Arch = 1 << 1; // ColdFire ISA_A

// -----------------------------------------------------------------------------

macro_rules! opcodes {
    {
        $(
            $name:ident $(. $suffix:ident )*
                ( $($bits:expr),+ ) ( $($mask:expr),+ )
                [ $( $($arg:tt):+ ),* ] $arch:expr ;
        )+
    } =>
    {
        static OPCODES: &'static [Opcode] = &[
            $(
                Opcode {
                    name: concat!(stringify!($name), $( ".", stringify!($suffix) )*),
                    size: size!($($suffix).*),
                    bits: words!($($bits),+),
                    mask: words!($($mask),+),
                    args: &[$(arg!($($arg):+)),*],
                    arch: $arch,
                },
            )+
        ];
    };
}

macro_rules! size {
    {   } => { Zero };
    { s } => { Byte };
    { b } => { Byte };
    { w } => { Word };
    { l } => { Long };
}

macro_rules! words {
    { $a:expr          } => { One($a    ) };
    { $a:expr, $b:expr } => { Two($a, $b) };
}

macro_rules! arg {
    { src    : $pos:expr } => { Arg::Modes(SRC,     $pos) };
    { dst    : $pos:expr } => { Arg::Modes(DST,     $pos) };
    { dstmem : $pos:expr } => { Arg::Modes(DST_MEM, $pos) };
    { modea  : $pos:expr } => { Arg::Modes(A,       $pos) };
    { data   : $pos:expr } => { Arg::DataReg($pos)        };
}

opcodes! {
//  NAME        WORDS             MASKS             OPERANDS            ARCHITECTURES
    add.l       (0xD080)          (0xF1C0)          [src:0, data:9]     CF_A;
    add.l       (0xD180)          (0xF1C0)          [data:9, dstmem:0]  CF_A;
    add.l       (0xD180)          (0xF1C0)          [data:9, modea:0]   CF_A|RELAX;

    move.b      (0x1000)          (0xF000)          [src:0, dst:6]      CF_A;
    move.w      (0x3000)          (0xF000)          [src:0, dst:6]      CF_A;
    move.l      (0x2000)          (0xF000)          [src:0, dst:6]      CF_A;

    muls.l      (0x4C00, 0x0400)  (0xFFC0, 0x8FFF)  [src:0, data:12]    CF_A;
    mulu.l      (0x4C00, 0x0000)  (0xFFC0, 0x8FFF)  [src:0, data:12]    CF_A;

    nop         (0x4E71)          (0xFFFF)          []                  CF_A; 
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name() {
        assert!(OPCODES.iter().any(|o| o.name == "move.b"));
        assert!(OPCODES.iter().any(|o| o.name == "nop"   ));
    }

    #[test]
    fn bits() {
        assert_eq!(opcode("nop").bits, One(0x4E71));
    }

    #[test]
    fn mask() {
        assert_eq!(opcode("nop").mask, One(0xFFFF));
    }

    fn opcode(name: &str) -> &Opcode {
        OPCODES.iter().find(|o| o.name == name).unwrap()
    }
}

