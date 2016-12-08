// Integer Type Specifiers
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

use num::{BigInt, Zero, One};

//use super::contains::Contains;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct IntSpec {
    pub store_width: u8,    // count of stored bits (== valued + ignored)
    pub value_width: u8,    // count of valued bits (<= stored)
    pub value_shift: u8,    // left shift of valued bits within stored bits
    pub signed:      bool,  // whether signed or unsigned
}

impl IntSpec {
    pub fn min_value(self) -> BigInt {
        if self.signed {
            BigInt::zero() - bit(self.value_width - 1)
        } else {
            BigInt::zero()
        }
    }

    pub fn max_value(self) -> BigInt {
        if self.signed {
            bit(self.value_width - 1) - BigInt::one()
        } else {
            bit(self.value_width    ) - BigInt::one()
        }
    }
}

impl /*Contains<BigInt> for*/ IntSpec {
    #[inline]
    fn contains(&self, value: &BigInt) -> Option<bool> {
        Some(
            *value >= self.min_value() &&
            *value <= self.max_value()
        )
    }
}

fn bit(n: u8) -> BigInt {
    BigInt::from(1 << (n as u64))
}

#[cfg(test)]
mod tests {
    use num::BigInt;
    use super::IntSpec;

    static U8: IntSpec = IntSpec {
        store_width: 16, value_width: 8, value_shift: 0, signed: false
    };

    static I8: IntSpec = IntSpec {
        store_width: 16, value_width: 8, value_shift: 0, signed: true
    };

    #[test]
    fn min_value() {
        assert_eq!( U8.min_value(), BigInt::from(   0) );
        assert_eq!( I8.min_value(), BigInt::from(-128) );
    }

    #[test]
    fn max_value() {
        assert_eq!( U8.max_value(), BigInt::from(255) );
        assert_eq!( I8.max_value(), BigInt::from(127) );
    }

    #[test]
    fn bit() {
        assert_eq!( super::bit(0), BigInt::from(1 << 0) );
        assert_eq!( super::bit(1), BigInt::from(1 << 1) );
        assert_eq!( super::bit(7), BigInt::from(1 << 7) );
    }
}

