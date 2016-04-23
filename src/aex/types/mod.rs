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

pub mod contains;
//pub mod builtin;
pub mod float;
pub mod int;
//pub mod res;

use std::rc::Rc;
use num::BigUint;

use aex::pos::Source;
use aex::types::float::FloatSpec;
use aex::types::int::IntSpec;

// Type expression

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Type<'a> {
    Ident   (Source<'a>, &'a str, Option<Rc<Type<'a>>>),
    Int     (Source<'a>, Option<IntSpec>),
    Float   (Source<'a>, Option<FloatSpec>),
    Array   (Source<'a>, Box<Type<'a>>, Option<BigUint>),
    Ptr     (Source<'a>, Box<Type<'a>>, Box<Type<'a>>),
    Struct  (Source<'a>, Vec<Member<'a>>),
    Union   (Source<'a>, Vec<Member<'a>>),
    Func    (Source<'a>, Vec<Member<'a>>, Vec<Member<'a>>),
}

// Complex type member

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Member<'a> (&'a str, Type<'a>);

// Basic equivalence and size information for a type
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum TypeForm {
    Inty    (Option<  IntSpec>),    // Int, Ptr
    Floaty  (Option<FloatSpec>),    // Float
    Opaque,                         // Array, Union, Struct, Func
}

impl<'a> Type<'a> {
    pub fn form(&self) -> TypeForm {
        match *self {
            Type::Ident  (_, _, Some(ref t))
                => t.form(),
            Type::Ident  (..)
                => panic!("Attempted to compute form of unresolved type identifier."),
            Type::Int    (_, s)          => TypeForm::Inty   (s),
            Type::Float  (_, s)          => TypeForm::Floaty (s),
            Type::Ptr    (_, ref t, _)   => t.form(),
            Type::Array  (..)            => TypeForm::Opaque,
            Type::Struct (..)            => TypeForm::Opaque,
            Type::Union  (..)            => TypeForm::Opaque,
            Type::Func   (..)            => TypeForm::Opaque,
        }
    }
}

