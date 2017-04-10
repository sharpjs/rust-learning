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

#[derive(Clone, Copy, Debug)]
pub struct Instruction {
    pub name:  &'static str,
    pub size:  Size,
    pub forms: &'static [Opcode],
}

#[derive(Clone, Copy, Debug)]
pub struct Opcode {
    pub bits: Words,
    pub mask: Words,
    pub args: &'static [Arg],
    pub arch: Arch,
}

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

    /// Immediate; 1 or 2 words after opwords
    Immediate,

    /// Quick immediate; 3 bits, 0 => 8
    Quick3(BitPos),

    /// Quick immediate; 8 bits, signed
    Quick8(BitPos),
}

pub type BitPos = u8;

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

// -----------------------------------------------------------------------------

/// Architecture flags.
pub type Arch = u16;

pub const RELAX: Arch = 1 << 0; // Relaxations enabled
pub const CF_A:  Arch = 1 << 1; // ColdFire ISA_A

// -----------------------------------------------------------------------------

macro_rules! opcodes {
    {
        $(
            $id:ident $name:ident $( . $suffix:ident )* {
                $(
                    ( $( $bits:expr   ),+ )
                    ( $( $mask:expr   ),+ )
                    [ $( $($arg:tt):+ ),* ]
                    $arch:expr ;
                )+
            }
        )*
    } =>
    {
        $(
            static $id: Instruction = Instruction {
                name: concat!(stringify!($name) $( , ".", stringify!($suffix) )*),
                size: size!($($suffix).*),
                forms: &[
                    $(
                        Opcode {
                            bits: words!($($bits),+),
                            mask: words!($($mask),+),
                            args: &[ $( arg!($($arg):+) ),* ],
                            arch: $arch,
                        },
                    )+
                ],
            };
        )*
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
    // Addressing mode combinations
    { daipmdxnfDXI : $pos:expr } => { Arg::Modes(D|A|AI|AP|AM|AD|AX|MS|ML|PD|PX|I, $pos) };
    { daipmdxnf___ : $pos:expr } => { Arg::Modes(D|A|AI|AP|AM|AD|AX|MS|ML        , $pos) };
    { __ipmdxnf___ : $pos:expr } => { Arg::Modes(    AI|AP|AM|AD|AX|MS|ML        , $pos) };

    // Other operand kinds
    { data : $pos:expr } => { Arg::DataReg($pos) };
    { addr : $pos:expr } => { Arg::AddrReg($pos) };
    { imm              } => { Arg::Immediate     };
    { q3   : $pos:expr } => { Arg::Quick3($pos)  };
}

opcodes! {
    //  WORDS             MASKS             OPERANDS                          ARCHITECTURES

    ADDAL adda.l {
        (0xD1C0)          (0xF1C0)          [daipmdxnfDXI:0, addr:9]          CF_A;
    }

    ADDIL addi.l {
        (0x0680)          (0xFFF8)          [imm, data:0]                     CF_A;
    }

    ADDQL addq.l {
        (0x5080)          (0xF1C0)          [q3:9, daipmdxnf___:0]            CF_A;
    }

    ADDL add.l {
      //use addq.l, addi.l, adda.l;
      //(0x5080)          (0xF1C0)          [q3:9, daipmdxnf___:0]            CF_A | RELAX; // -> addq.l
      //(0x0680)          (0xFFF8)          [imm, data:0]                     CF_A | RELAX; // -> addi.l
      //(0xD1C0)          (0xF1C0)          [daipmdxnfDXI:0, addr:9]          CF_A | RELAX; // -> adda.l
        (0xD080)          (0xF1C0)          [daipmdxnfDXI:0, data:9]          CF_A;
        (0xD180)          (0xF1C0)          [data:9, __ipmdxnf___:0]          CF_A;
    }

    ADDXL addx.l {
        (0xD180)          (0xF1F8)          [data:0, data:9]                  CF_A;
    }

    MOVEB move.b {
        (0x1000)          (0xF000)          [daipmdxnfDXI:0, daipmdxnf___:6]  CF_A;
    }

    MOVEW move.w {
        (0x3000)          (0xF000)          [daipmdxnfDXI:0, daipmdxnf___:6]  CF_A;
    }

    MOVEL move.l {
        (0x2000)          (0xF000)          [daipmdxnfDXI:0, daipmdxnf___:6]  CF_A;
    }

    MULSL muls.l {
        (0x4C00, 0x0400)  (0xFFC0, 0x8FFF)  [daipmdxnfDXI:0, data:12]         CF_A;
    }

    MULUL mulu.l {
        (0x4C00, 0x0000)  (0xFFC0, 0x8FFF)  [daipmdxnfDXI:0, data:12]         CF_A;
    }

    NOP nop {
        (0x4E71)          (0xFFFF)          []                                CF_A; 
    }
}

// For    assembly : name -> [opcode] -- use  first opcode that matches
// For disassembly : [(opcode, name)] -- find first opcode that matches, then get name
//
//   "movea.l" -> [a]       -- a    is  the opcode  for movea.l
//   "move.l"  -> [b, c]    -- b, c are the opcodes for move.l
//
// for 0x2...:
//   [ (a, "movea.l")
//   , (b, "move.l" )
//   , (c, "move.l" )
//   ]
//

// First level disasm trie:
//
// 0 -> ori|btst|bchg|bset|andi|subi|addi|eori|cmpi
// 1 -> move.b
// 2 -> move.l|movea.l
// 3 -> move.w|movea.w
// 4 -> (everything else!)
// 5 -> addq|scc|subq|tpf
// 6 -> bra|bsr|bcc
// 7 -> moveq
// 8 -> or|divu.w|divs.w
// 9 -> sub|subx|suba
// A -> (mac stuff)
// B -> cmp|cmpa|eor
// C -> and|mulu.w|muls.wA
// D -> add|addx|adda
// E -> asl|asr|lksl|lsr
// F -> cpushl|wddata|wdebug|(fp stuff)

pub fn decode(value: u16) -> Option<&'static Instruction> {
    match value >> 12 {
        0x0 => Some(&ADDIL),
        0x1 => Some(&MOVEB),
        0x2 => Some(&MOVEL),
        0x3 => Some(&MOVEW),
        _   => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name() {
        assert_eq!(MOVEB.name, "move.b");
        assert_eq!(NOP  .name, "nop");
    }

    #[test]
    fn bits() {
        assert_eq!(NOP.forms[0].bits, One(0x4E71));
    }

    #[test]
    fn mask() {
        assert_eq!(NOP.forms[0].mask, One(0xFFFF));
    }
}

