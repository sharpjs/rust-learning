// ColdFire Index Registers
//
// This file is part of AEx.
// Copyright (C) 2016 Jeffrey Sharp
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

use std::fmt::{self, Formatter};

use aex::asm::{AsmDisplay, AsmStyle};
use super::{AddrReg, DataReg};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Index {
    Data(DataReg),
    Addr(AddrReg),
}

impl AsmDisplay for Index {
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle) -> fmt::Result {
        match *self {
            Index::Data(ref r) => r.fmt(f, s),
            Index::Addr(ref r) => r.fmt(f, s),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum Scale {
    Byte = 1,
    Word = 2,
    Long = 4,
}

impl AsmDisplay for Scale {
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle) -> fmt::Result {
        s.write_scale(f, *self as u8)
    }
}

