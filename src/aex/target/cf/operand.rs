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
    Unary([(Operand, BitPos); 1]),

    /// Two operands.
    Binary([(Operand, BitPos); 2]),

    /// Three operands.
    Ternary([(Operand, BitPos); 3]),

    // SpecialFormA,
    // SpecialFormB,
    // ...
}

/// Operand forms.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Operand {
    /// Modes daipmdxDXnfI (any) (6 bits)
    AnyMode,

    /// Modes d_ipmdxDXnfI (any except addr reg) (6 bits)
    DataMode,

    /// Modes daipmdx__nf_ (mutable) (6 bits)
    MutMode,

    /// Modes __ipmdx__nf_ (mutable memory) (6 bits)
    MutMemMode,

    /// Data register (3 bits)
    DataReg,

    /// Address register (3 bits)
    AddrReg,

    /// Data or address register (4 bits)
    NormalReg,

    /// Control register (12 bits)
    CtlReg,

    /// Debug control register (5 bits)
    DbgReg,

    /// Condition code register (implicit)
    Ccr,

    /// Condition code register (implicit)
    Sr,

    /// Data/address register list (16 bits in extension word)
    RegList,

    /// Condition code (4 bits),
    Cond,

    /// Cache selector (2 bits)
    CacheSel,

    /// Immediate (16 or 32 bits in extension words)
    Immediate,

    /// Quick immediate (3 bits; 0 => 8)
    Quick3,

    /// Quick immediate (8 bits signed)
    Quick8,
}

