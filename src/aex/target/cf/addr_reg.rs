// ColdFire Address Registers
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

use aex::ast::Reg;
use aex::fmt::ToCode;

pub use self::AddrReg::*;

/// ColdFire address registers.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum AddrReg {
    A0, A1, A2, A3, A4, A5, FP, SP
}

static ADDR_REGS: [AddrReg; 8] = [
    A0, A1, A2, A3, A4, A5, FP, SP
];

static ADDR_REG_NAMES: [&'static str; 8] = [
    "a0", "a1", "a2", "a3", "a4", "a5", "fp", "sp"
];

pub const A6: AddrReg = FP;
pub const A7: AddrReg = SP;

impl AddrReg {
    /// Returns the address register with the given number.
    #[inline]
    pub fn with_num(n: u8) -> Self {
        ADDR_REGS[n as usize]
    }

    /// Returns the number of the address register.
    #[inline]
    pub fn num(self) -> u8 {
        self as u8
    }

    /// Returns the name of the address register.
    #[inline]
    pub fn name(self) -> &'static str {
        ADDR_REG_NAMES[self as usize]
    }
}

impl<A> ToCode<A> for AddrReg {
    type Output = Reg<'static, A>;

    /// Converts to a code-formattable value with the given annotation.
    #[inline]
    fn to_code(&self, ann: A) -> Self::Output {
        Reg::new_with_ann(self.name(), ann)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_num() {
        assert_eq!( AddrReg::with_num(3), A3 );
    }

    #[test]
    fn num() {
        assert_eq!( FP.num(), 6 );
    }

    #[test]
    fn name() {
        assert_eq!( A5.name(), "a5" );
    }

    #[test]
    fn to_code() {
        let c = A0.to_code(42);

        assert_eq!(c.name, "a0");
        assert_eq!(c.ann,  42  );
    }
}

