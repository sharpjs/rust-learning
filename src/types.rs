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

use std::rc::Rc;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Type {
    Int     { value_width: Option<u8>
            , store_width: Option<u8>
            , signed:      Option<bool>
            },

    Array   { elem_type: Box<Type>
            , length:    Option<u64>
            },

    Ptr     { addr_type: Box<Type>
            , val_type:  Box<Type>
            },

    Struct  { members: Vec<Member>
            },

    Union   { members: Vec<Member>
            },

    Func    { params:  Vec<Member>
            , returns: Vec<Member>
            },
}
use self::Type::*;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Member {
    name: Rc<String>,
    ty:   Box<Type>,
}

pub const U8:  &'static Type = &Int { value_width: Some( 8), store_width: Some( 8), signed: Some(false) };
pub const U16: &'static Type = &Int { value_width: Some(16), store_width: Some(16), signed: Some(false) };
pub const U32: &'static Type = &Int { value_width: Some(32), store_width: Some(32), signed: Some(false) };
pub const U64: &'static Type = &Int { value_width: Some(64), store_width: Some(64), signed: Some(false) };

pub const I8:  &'static Type = &Int { value_width: Some( 8), store_width: Some( 8), signed: Some(true) };
pub const I16: &'static Type = &Int { value_width: Some(16), store_width: Some(16), signed: Some(true) };
pub const I32: &'static Type = &Int { value_width: Some(32), store_width: Some(32), signed: Some(true) };
pub const I64: &'static Type = &Int { value_width: Some(64), store_width: Some(64), signed: Some(true) };

// Abstract integer
pub const INT: &'static Type = &Int {
    value_width: None,
    store_width: None,
    signed:      None,
};

impl Type {
    pub fn is_scalar(&self) -> bool {
        is!(*self => Int { .. })
    }

    pub fn value_width(&self) -> Option<u8> {
        match *self {
            Int { value_width, .. } => value_width,
            _                       => None
        }
    }

    pub fn store_width(&self) -> Option<u8> {
        match *self {
            Int { store_width, .. } => store_width,
            _                       => None
        }
    }

    //pub fn contains(&self, value: Value) -> bool {
    //}
}

