// Types
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

mod int;
mod float;

//use num::{BigInt, BigUint, Zero, One};
//
//use self::Type::*;
//
//#[derive(Clone, Hash, Eq, PartialEq, Debug)]
//pub enum Type<'a> {
//    Ref    (&'a str),
//    Int    (Option<int::IntSpec>),
//    Float  (Option<float::FloatSpec>),
//    Array  (Box<Type<'a>>, Option<BigUint>),
//    Ptr    (Box<Type<'a>>, Box<Type<'a>>),
//    Struct (Vec<Member<'a>>),
//    Union  (Vec<Member<'a>>),
//    Func   (Vec<Member<'a>>, Vec<Member<'a>>),
//}
//
//#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
//pub struct IntSpec {
//    pub value_width: u8,    // count of value bits
//    pub store_width: u8,    // count of value + padding bits
//    pub signed:      bool,  // whether signed or unsigned
//}
//
//#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
//pub struct FloatSpec {
//    pub value_width: u8,    // count of value bits
//    pub store_width: u8,    // count of value + padding bits
//}
//
//#[derive(Clone, Hash, Eq, PartialEq, Debug)]
//pub struct Member<'a> {
//    pub name: &'a str,
//    pub ty:   Type<'a>,
//}
//
//// Shorthand for built-in types
//pub type StaticType = &'static Type<'static>;
//
//// Abstract Integer
//pub const INT: StaticType = &Int(None);
//
//// Concrete Integers
//macro_rules! ints {
//    ($($name:ident: $vw:expr, $sw:expr, $sg:expr;)*) => {$(
//        pub const $name: StaticType = &Int(Some(IntSpec {
//            value_width: $vw, store_width: $sw, signed: $sg
//        }));
//    )*}
//}
//ints! {
//     U8:  8,  8, false;
//    U16: 16, 16, false;
//    U32: 32, 32, false;
//    U64: 64, 64, false;
//
//     I8:  8,  8, true;
//    I16: 16, 16, true;
//    I32: 32, 32, true;
//    I64: 64, 64, true;
//}
//
//impl<'a> Type<'a> {
//    pub fn is_scalar(&self) -> bool {
//        is!(*self => Int(_))
//    }
//
//    pub fn value_width(&self) -> Option<u8> {
//        match *self {
//            Int(Some(IntSpec { value_width, .. })) => Some(value_width),
//            _                                      => None
//        }
//    }
//
//    pub fn store_width(&self) -> Option<u8> {
//        match *self {
//            Int(Some(IntSpec { store_width, .. })) => Some(store_width),
//            _                                      => None
//        }
//    }
//}
//
//impl IntSpec {
//    pub fn min_value(self) -> BigInt {
//        if self.signed {
//            BigInt::zero() - bit(self.value_width - 1)
//        } else {
//            BigInt::zero()
//        }
//    }
//
//    pub fn max_value(self) -> BigInt {
//        if self.signed {
//            bit(self.value_width - 1) - BigInt::one()
//        } else {
//            bit(self.value_width    ) - BigInt::one()
//        }
//    }
//}
//
//fn bit(n: u8) -> BigInt {
//    BigInt::from(1 << (n as u64))
//}
//
//#[cfg(test)]
//mod tests {
//    use num::BigInt;
//
//    mod int_spec {
//        use num::BigInt;
//        use super::super::IntSpec;
//
//        static U8: IntSpec = IntSpec {
//            value_width: 8, store_width: 16, signed: false
//        };
//
//        static I8: IntSpec = IntSpec {
//            value_width: 8, store_width: 16, signed: true
//        };
//
//        #[test]
//        fn min_value() {
//            assert_eq!( U8.min_value(), BigInt::from(   0) );
//            assert_eq!( I8.min_value(), BigInt::from(-128) );
//        }
//
//        #[test]
//        fn max_value() {
//            assert_eq!( U8.max_value(), BigInt::from(255) );
//            assert_eq!( I8.max_value(), BigInt::from(127) );
//        }
//    }
//
//    #[test]
//    fn bit() {
//        assert_eq!( super::bit(0), BigInt::from(1 << 0) );
//        assert_eq!( super::bit(1), BigInt::from(1 << 1) );
//        assert_eq!( super::bit(7), BigInt::from(1 << 7) );
//    }
//}
//
