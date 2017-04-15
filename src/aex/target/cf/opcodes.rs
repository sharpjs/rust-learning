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

use super::{Mnemonic, Operands, Size};
use super::Mnemonic::*;
use super::Operands::*;
use super::Operand::*;
use super::Size::*;

// -----------------------------------------------------------------------------

/// An entry in the opcodes table.
///
/// Describes how to assemble or disassemble an instruction, along with the
/// supported argument types and architectures.
///
#[derive(Clone, Copy, Debug)]
pub struct Opcode {
    /// Instruction name.
    pub name: Mnemonic,                     // 1 byte (why not padded?)

    /// Operation size.
    pub size: Size,                         // 1 byte (why not padded?)

    /// Opcode bits.
    pub bits: (u16, u16),                   // 4 bytes

    /// Mask of significant opcode bits.
    pub mask: (u16, u16),                   // 4 bytes

    /// Operand set.
    pub args: Operands,                     // 7 bytes + 1 pad

    /// Flags: architectures, etc.
    pub flags: Flags,                       // 2 bytes
}

/*
// -----------------------------------------------------------------------------

/// Addressing mode flags.
pub type Modes = u16;

pub const DR: Modes = 1 <<  0; // 0.*: data reg direct
pub const AR: Modes = 1 <<  1; // 1.*: addr reg direct
pub const AI: Modes = 1 <<  2; // 2.*: addr reg indirect
pub const AP: Modes = 1 <<  3; // 3.*: addr reg indirect, auto-increment (plus)
pub const AM: Modes = 1 <<  4; // 4.*: addr reg indirect, auto-decrement (minus)
pub const AD: Modes = 1 <<  5; // 5.*: addr reg indirect, displaced
pub const AX: Modes = 1 <<  6; // 6.*: addr reg indirect, indexed, displaced
pub const MS: Modes = 1 <<  7; // 7.0: absolute short
pub const ML: Modes = 1 <<  8; // 7.1: absolute long
pub const PD: Modes = 1 <<  9; // 7.2: pc-relative, displaced
pub const PX: Modes = 1 << 10; // 7.3: pc-relative, indexed, displaced
pub const IM: Modes = 1 << 11; // 7.4: immediate
*/

// -----------------------------------------------------------------------------

/// Opcode flags.
pub type Flags = u16;

pub const EXT_WORD:  Flags = 1 << 0; // Has extension word
pub const CF_A:      Flags = 1 << 1; // Appears in ColdFire ISA_A
pub const CF_A2:     Flags = 1 << 2; // Appears in ColdFire ISA_A+
pub const CF_B:      Flags = 1 << 3; // Appears in ColdFire ISA_B
pub const CF_C:      Flags = 1 << 4; // Appears in ColdFire ISA_C
pub const CF_FPU:    Flags = 1 << 5; // Appears in ColdFire FPU
pub const CF_MAC:    Flags = 1 << 6; // Appears in ColdFire MAC
pub const CF_EMAC:   Flags = 1 << 7; // Appears in ColdFire EMAC
pub const CF_EMAC_B: Flags = 1 << 8; // Appears in ColdFire EMAC_B

pub const CF_A_UP:   Flags = CF_A | CF_A2 | CF_B | CF_C;
pub const CF_A2_UP:  Flags =        CF_A2 | CF_B | CF_C;
pub const CF_B_UP:   Flags =                CF_B | CF_C;

// -----------------------------------------------------------------------------

macro_rules! opcodes {
    {
        $(
            $name:ident $size:tt ( $($bits:expr),+ ) ( $($mask:expr),+ )
                [ $( $($arg:tt):+ ),* ] $flags:expr ;
        )*
    } =>
    {
        pub static OPCODES: &'static [Opcode] = &[
            $(
                Opcode {
                    name:  $name,
                    size:  size!($size),
                    bits:  words!($($bits),+),
                    mask:  words!($($mask),+),
                    args:  args!($( $($arg):+ ),*),
                    flags: $flags | ext!($($bits),+),
                },
            )*
        ];
    };
}

macro_rules! size {
    { - } => { Zero };
    { S } => { Byte };
    { B } => { Byte };
    { W } => { Word };
    { L } => { Long };
}

macro_rules! words {
    { $a:expr          } => { ($a,  0) };
    { $a:expr, $b:expr } => { ($a, $b) };
}

macro_rules! ext {
    { $a:expr          } => { 0        };
    { $a:expr, $b:expr } => { EXT_WORD };
}

macro_rules! args {
    { }                      => { Nullary };

    { $k0:ident : $p0:expr } => { Unary   ([ (arg!($k0), $p0) ]) };

    { $k0:ident : $p0:expr , 
      $k1:ident : $p1:expr } => { Binary  ([ (arg!($k0), $p0) ,
                                             (arg!($k1), $p1) ]) };

    { $k0:ident : $p0:expr , 
      $k1:ident : $p1:expr ,
      $k2:ident : $p2:expr } => { Ternary ([ (arg!($k0), $p0) ,
                                             (arg!($k1), $p1) ,
                                             (arg!($k2), $p2) ]) };
}

macro_rules! arg {
    // Addressing mode combinations
    { daipmdxnfDXI } => { AnyMode };
    { daipmdxnf___ } => { MutMode };
    { __ipmdxnf___ } => { MutMemMode };

    // Other operand kinds
    { data } => { DataReg   };
    { addr } => { AddrReg   };
    { imm  } => { Immediate };
    { q3   } => { Quick3    };
}

opcodes! {
//  NAME    S  WORDS             MASKS             OPERANDS                          FLAGS
//  ------  -  ----------------  ----------------  --------------------------------  -----
    Adda    L  (0xD1C0)          (0xF1C0)          [daipmdxnfDXI:0, addr:9]          CF_A_UP;

    Addi    L  (0x0680)          (0xFFF8)          [imm:0, data:0]                   CF_A_UP;

    Addq    L  (0x5080)          (0xF1C0)          [q3:9, daipmdxnf___:0]            CF_A_UP;

    Add     L  (0xD080)          (0xF1C0)          [daipmdxnfDXI:0, data:9]          CF_A_UP;
    Add     L  (0xD180)          (0xF1C0)          [data:9, __ipmdxnf___:0]          CF_A_UP;

    Addx    L  (0xD180)          (0xF1F8)          [data:0, data:9]                  CF_A_UP;

    Move    B  (0x1000)          (0xF000)          [daipmdxnfDXI:0, daipmdxnf___:6]  CF_A_UP;

    Move    W  (0x3000)          (0xF000)          [daipmdxnfDXI:0, daipmdxnf___:6]  CF_A_UP;

    Move    L  (0x2000)          (0xF000)          [daipmdxnfDXI:0, daipmdxnf___:6]  CF_A_UP;

    Muls    L  (0x4C00, 0x0400)  (0xFFC0, 0x8FFF)  [daipmdxnfDXI:0, data:12]         CF_A_UP;

    Mulu    L  (0x4C00, 0x0000)  (0xFFC0, 0x8FFF)  [daipmdxnfDXI:0, data:12]         CF_A_UP;

    Nop     -  (0x4E71)          (0xFFFF)          []                                CF_A_UP; 
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

// Trie in structure, or ad-hoc trie in code (bunch o'matches)?
//   - code: does not require mask in opcodes table any more
//   - code: uses I cache instead of D cache
//   - data: automatic generation
//   - data: easier evolution
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

