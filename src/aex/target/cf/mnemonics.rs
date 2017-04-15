// ColdFire Instruction Names
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

use self::Mnemonic::*;

/// Instruction names.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Mnemonic {
    Add, Adda, Addi, Addq, Addx,
    Move,
    Muls, Mulu,
    Nop,
}

impl Mnemonic {
    /// Returns the string representation of the instruction name.
    pub fn as_str(self) -> &'static str {
        match self {
            Add  => "add",
            Adda => "adda",
            Addi => "addi",
            Addq => "addq",
            Addx => "addx",
            Move => "move",
            Muls => "muls",
            Mulu => "mulu",
            Nop  => "nop",
        }
    }
}

