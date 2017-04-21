// ColdFire Data Registers
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

pub use self::DataReg::*;

/// ColdFire data registers.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum DataReg {
    D0, D1, D2, D3, D4, D5, D6, D7
}

static DATA_REGS: [DataReg; 8] = [
    D0, D1, D2, D3, D4, D5, D6, D7
];

static DATA_REG_NAMES: [&'static str; 8] = [
    "d0", "d1", "d2", "d3", "d4", "d5", "d6", "d7"
];

impl DataReg {
    /// Returns the data register with the given number.
    #[inline]
    pub fn with_num(n: u8) -> Self {
        DATA_REGS[n as usize]
    }

    /// Returns the number of the data register.
    #[inline]
    pub fn num(self) -> u8 {
        self as u8
    }

    /// Returns the name of the data register.
    #[inline]
    pub fn name(self) -> &'static str {
        DATA_REG_NAMES[self as usize]
    }
}

impl<A> ToCode<A> for DataReg {
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
        assert_eq!( DataReg::with_num(3), D3 );
    }

    #[test]
    fn num() {
        assert_eq!( D6.num(), 6 );
    }

    #[test]
    fn name() {
        assert_eq!( D5.name(), "d5" );
    }

    #[test]
    fn to_code() {
        let c = D0.to_code(42);

        assert_eq!(c.name, "d0");
        assert_eq!(c.ann,   42 );
    }
}

