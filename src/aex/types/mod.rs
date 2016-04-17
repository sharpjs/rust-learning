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

pub mod int;
pub mod float;

//// Shorthand for built-in types
//pub type StaticType = &'static Type<'static>;

//// Abstract Integer
//pub const INT: StaticType = &Int(None);

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

