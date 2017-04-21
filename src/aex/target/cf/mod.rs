// ColdFire Target
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

mod decode;
mod decode_context;
mod mnemonics;
mod opcodes;
mod operand;

mod index;

mod data_reg; // mode 0
mod addr_reg; // mode 1

pub use self::decode::*;
pub use self::decode_context::*;
pub use self::mnemonics::*;
pub use self::opcodes::*;
pub use self::operand::*;

pub use self::index::*;

pub use self::data_reg::*; // mode 0
pub use self::addr_reg::*; // mode 1

//mod addr_disp;
//mod addr_disp_idx;
//mod misc_regs;
//mod pc_disp;
//mod pc_disp_idx;
//mod scale;
//mod value;

//pub use self::addr_disp::*;
//pub use self::addr_disp_idx::*;
//pub use self::data_reg::*;
//pub use self::misc_regs::*;
//pub use self::pc_disp::*;
//pub use self::pc_disp_idx::*;
//pub use self::scale::*;
//pub use self::value::*;

/// Operation sizes.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Size {
    /// No associated size.
    Zero,

    /// Byte
    Byte,

    /// Word (2 bytes)
    Word,

    /// Longword (4 bytes)
    Long,
}

