// Type System
//
// This file is part of AEx.
// Copyright (C) 2015 Jeffrey Sharp
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

use self::Type::*;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct IntSpec {
    value_width: u8,    // count of value bits
    store_width: u8,    // count of value + padding bits
    signed:      bool,  // whether signed or unsigned
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Type<'a> {
    Ref    (&'a str),
    Int    (Option<IntSpec>),
    Array  (&'a Type<'a>, Option<u64>),
    Ptr    (&'a Type<'a>, &'a Type<'a>),
    Struct (Vec<Member<'a>>),
    Union  (Vec<Member<'a>>),
    Func   (Vec<Member<'a>>, Vec<Member<'a>>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Member<'a> {
    name: &'a str,
    ty:   &'a Type<'a>,
}

// Abstract Integer
pub const INT: &'static Type<'static> = &Int(None);

// Concrete Integers
macro_rules! ints {
    ($($name:ident: $vw:expr, $sw:expr, $sg:expr;)*) => {$(
        pub const $name: &'static Type<'static> = &Int(Some(IntSpec {
            value_width: $vw, store_width: $sw, signed: $sg
        }));
    )*}
}
ints! {
     U8:  8,  8, false;
    U16: 16, 16, false;
    U32: 32, 32, false;
    U64: 64, 64, false;

     I8:  8,  8, true;
    I16: 16, 16, true;
    I32: 32, 32, true;
    I64: 64, 64, true;
}

impl<'a> Type<'a> {
    pub fn is_scalar(&self) -> bool {
        is!(*self => Int(_))
    }

    pub fn value_width(&self) -> Option<u8> {
        match *self {
            Int(Some(IntSpec { value_width, .. })) => Some(value_width),
            _                                      => None
        }
    }

    pub fn store_width(&self) -> Option<u8> {
        match *self {
            Int(Some(IntSpec { store_width, .. })) => Some(store_width),
            _                                      => None
        }
    }

    //pub fn contains(&self, value: Value) -> bool {
    //}
}

