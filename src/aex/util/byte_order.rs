// Byte Order Conversion
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

use std::mem::transmute;

/// Specifies the order of bytes within an encoded numeric value.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ByteOrder {
    /// Little-endian byte order: least to most significant.
    LittleEndian,
    /// Big-endian byte order: most to least significant.
    BigEndian,
}

pub use self::ByteOrder::{
    LittleEndian as LE,
    BigEndian    as BE
};

pub trait Endian {
    /// Converts `self` from the target's byte order to the given byte order.
    fn to_order(self, ord: ByteOrder) -> Self;

    /// Converts a value from the given byte order to the target's byte order.
    fn from_order(ord: ByteOrder, x: Self) -> Self;
}

macro_rules! impl_endian_int {
    { $( $t:ident ),* } => {
        $(
            /// Converts `self` from the target's byte order to the given byte order.
            impl Endian for $t {
                fn to_order(self, order: ByteOrder) -> Self {
                    match order {
                        LE => self.to_le(),
                        BE => self.to_be(),
                    }
                }

                /// Converts a value from the given byte order to the target's byte order.
                fn from_order(order: ByteOrder, x: Self) -> Self {
                    match order {
                        LE => $t::from_le(x),
                        BE => $t::from_be(x),
                    }
                }
            }
        )*
    }
}

macro_rules! impl_endian_float {
    { $( $t:ident : $i:ident ),* } => {
        $(
            /// Converts `self` from the target's byte order to the given byte order.
            impl Endian for $t {
                fn to_order(self, order: ByteOrder) -> Self {
                    let mut i: $i = unsafe { transmute(self) };
                    i = i.to_order(order);
                    unsafe { transmute(i) }
                }

                /// Converts a value from the given byte order to the target's byte order.
                fn from_order(order: ByteOrder, x: Self) -> Self {
                    let mut i: $i = unsafe { transmute(x) };
                    i = $i::from_order(order, i);
                    unsafe { transmute(i) }
                }
            }
        )*
    }
}

impl_endian_int! { u16, i16, u32, i32, u64, i64, usize, isize }

impl_endian_float! { f32:u32, f64:u64 }

#[cfg(test)]
mod tests {
    use std::mem::transmute;
    use super::*;

    #[test]
    fn to_order_int() {
        let be: u16 = 0x1234.to_order(BE);
        let le: u16 = 0x1234.to_order(LE);

        assert_eq!(be.swap_bytes(), le);
    }

    #[test]
    fn from_order_int() {
        let be: u16 = u16::from_order(BE, 0x1234);
        let le: u16 = u16::from_order(LE, 0x1234);

        assert_eq!(be.swap_bytes(), le);
    }

    #[test]
    fn to_order_float() {
        let be: f32 = 12.34.to_order(BE);
        let le: f32 = 12.34.to_order(LE);

        let be: u32 = unsafe { transmute(be) };
        let le: u32 = unsafe { transmute(le) };

        assert_eq!(be.swap_bytes(), le);
    }

    #[test]
    fn from_order_float() {
        let be: f32 = f32::from_order(BE, 12.34);
        let le: f32 = f32::from_order(LE, 12.34);

        let be: u32 = unsafe { transmute(be) };
        let le: u32 = unsafe { transmute(le) };

        assert_eq!(be.swap_bytes(), le);
    }
}

