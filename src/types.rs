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

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Type {
    Int     { value_width: u8, store_width: u8, signed: bool },
    Array   { elem_type: Box<Type>, length: Option<u64> },
    Ptr     { addr_type: Box<Type>, val_type: Box<Type> },
    Struct  { members: Vec<Member> },
    Union   { members: Vec<Member> },
    Func    { params: Vec<Member>, returns: Vec<Member> },
}
use self::Type::*;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Member {
    name:   String,
    type_:  Box<Type>,
}

//#[derive(Clone, Eq, PartialEq, Debug)]
//pub enum Value {
//    Int:    u64
//}

pub const U8:  &'static Type = &Int { value_width:  8, store_width:  8, signed: false };
pub const U16: &'static Type = &Int { value_width: 16, store_width: 16, signed: false };
pub const U32: &'static Type = &Int { value_width: 32, store_width: 32, signed: false };
pub const U64: &'static Type = &Int { value_width: 64, store_width: 64, signed: false };

pub const I8:  &'static Type = &Int { value_width:  8, store_width:  8, signed: true };
pub const I16: &'static Type = &Int { value_width: 16, store_width: 16, signed: true };
pub const I32: &'static Type = &Int { value_width: 32, store_width: 32, signed: true };
pub const I64: &'static Type = &Int { value_width: 64, store_width: 64, signed: true };

impl Type {
    pub fn is_scalar(&self) -> bool {
        is!(*self => Int { .. })
    }

    pub fn value_width(&self) -> Option<u8> {
        match *self {
            Int { value_width, .. } => Some(value_width),
            _                       => None
        }
    }

    pub fn store_width(&self) -> Option<u8> {
        match *self {
            Int { store_width, .. } => Some(store_width),
            _                       => None
        }
    }

    //pub fn contains(&self, value: Value) -> bool {
    //}
}

