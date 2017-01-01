// Integer Type Specifiers
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

use num::{BigInt, Zero, One};

//use super::contains::Contains;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct IntSpec {
    pub store_width: u8,    // count of stored bits (== valued + ignored)
    pub value_width: u8,    // count of valued bits (<= stored)
    pub value_scale: u8,    // distance of value lsb from storage lsb
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

    pub fn encode(self, value: &BigInt) -> Result<u64, ()> {
        use num::ToPrimitive;

        if self.contains(value) != Some(true) { return Err(()) }

        let mut v;
        if self.signed {
            v = value.to_i64().unwrap() as u64;
            v &= 1u64
                .checked_shl(self.value_width as u32).unwrap_or(0)
                .wrapping_sub(1);
        } else {
            v = value.to_u64().unwrap();
        }

        v = v.checked_shl(self.value_scale as u32).unwrap_or(0);

        Ok(v)
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
        store_width: 16, value_width: 8, value_scale: 3, signed: false
    };

    static I8: IntSpec = IntSpec {
        store_width: 16, value_width: 8, value_scale: 3, signed: true
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

    #[test]
    fn encode_overflow() {
        let v = BigInt::from(0x100);
        assert_eq!( U8.encode(&v), Err(()) );
    }

    #[test]
    fn encode_unsigned() {
        let v = BigInt::from(0x93);
        assert_eq!( U8.encode(&v), Ok(0x498) );
        // 0b_0000_0000_1001_0011 = 0x0093 input
        // 0b_0000_0100_1001_1000 = 0x0498 shifted by value scale
    }

    #[test]
    fn encode_signed() {
        let v = BigInt::from(-7);
        assert_eq!( I8.encode(&v), Ok(0x7C8) );
        // 0b_0000_0000_0000_0111 =     -7 input
        // 0b_1111_1111_1111_1001 = 0xFFF9 coerced to unsigned
        // 0b_0000_0000_1111_1001 = 0x00F9 masked  by value width
        // 0b_0000_0111_1100_1000 = 0x07C8 shifted by value scale
    }
}

