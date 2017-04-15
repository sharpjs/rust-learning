// ColdFire Operands
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

use aex::util::BitPos;

/// Operand combinations.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Operands {
    /// No operands.
    Nullary,

    /// One operand.
    Unary([Operand; 1]),

    /// Two operands.
    Binary([Operand; 2]),

    /// Three operands.
    Ternary([Operand; 3]),

    // SpecialFormA,
    // SpecialFormB,
    // ...
}

/// Operand forms.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Operand {
    /// Modes daipmdxDXnfI (any) (6 bits)
    AnyMode(BitPos),

    /// Modes d_ipmdxDXnfI (any except addr reg) (6 bits)
    DataMode(BitPos),

    /// Modes daipmdx__nf_ (mutable) (6 bits)
    MutMode(BitPos),

    /// Modes __ipmdx__nf_ (mutable memory) (6 bits)
    MutMemMode(BitPos),

    /// Data register (3 bits)
    DataReg(BitPos),

    /// Address register (3 bits)
    AddrReg(BitPos),

    /// Data or address register (4 bits)
    NormalReg(BitPos),

    /// Control register (12 bits)
    CtlReg(BitPos),

    /// Debug control register (5 bits)
    DbgReg(BitPos),

    /// Condition code register (implicit)
    Ccr,

    /// Condition code register (implicit)
    Sr,

    /// Data/address register list (16 bits in extension word)
    RegList,

    /// Condition code (4 bits),
    Cond(BitPos),

    /// Cache selector (2 bits)
    CacheSel(BitPos),

    /// Immediate (16 or 32 bits in extension words)
    Immediate,

    /// Quick immediate (3 bits; 0 => 8)
    Quick3(BitPos),

    /// Quick immediate (8 bits signed)
    Quick8(BitPos),
}

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

