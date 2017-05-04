// ColdFire Operand Forms
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

//use std::io;
use aex::util::BitPos;

use super::DecodeRead;

use super::OperandForms::*;
use super::OperandForm::*;

/// Operand form combinations.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OperandForms {
    /// No operands.
    Nullary,

    /// One operand.
    Unary([OperandForm; 1]),

    /// Two operands.
    Binary([OperandForm; 2]),

    /// Three operands.
    Ternary([OperandForm; 3]),

    // SpecialFormA,
    // SpecialFormB,
    // ...
}

impl OperandForms {
    pub fn decode<R: DecodeRead>(self, c: &mut R) -> bool {
        match self {
            Nullary       => true,
            Unary(opds)   => opds[0].decode(c),
            Binary(opds)  => opds[0].decode(c) &&
                             opds[1].decode(c),
            Ternary(opds) => opds[0].decode(c) &&
                             opds[1].decode(c) &&
                             opds[2].decode(c),
        }
    }
}

/// Operand forms.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OperandForm {
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


impl OperandForm {
    pub fn decode<R: DecodeRead>(self, c: &mut R) -> bool {
        match self {
            AnyMode(pos) => {
                true
            },
            DataMode(pos) => {
                // decode mode
                // check if mode allowed here
                // return expr
                true
            },
            _ => false,
        }
    }
}

