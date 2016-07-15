// Operators
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

pub mod dispatch;

use std::borrow::Borrow;
use std::collections::HashMap;
//use std::fmt::{self, Debug, Formatter};

//use aex::value::Value;

use self::Assoc::*;
use self::Arity::*;

// Temporary
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Context;

// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OperatorTable {
    map: HashMap<&'static str, OperatorEntry>,
}

impl OperatorTable {
    pub fn new() -> Self {
        OperatorTable { map: HashMap::new() }
    }

    pub fn add(&mut self, op: Operator) {
        let entry = self.map
            .entry(op.chars)
            .or_insert_with(|| OperatorEntry::new());

        match (op.arity, op.assoc) {
            (Unary /*(..)*/, Right) => entry.prefix = Some(op),
            _                      => entry.suffix = Some(op),
        }
    }

    pub fn map(&self) -> &HashMap<&'static str, OperatorEntry> {
        &self.map
    }

    pub fn get<S>(&self, chars: S) -> Option<&OperatorEntry>
    where S: Borrow<str> {
        self.map.get(chars.borrow())
    }
}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct OperatorEntry {
    pub prefix: Option<Operator>,   // prefix
    pub suffix: Option<Operator>,   // infix or postfix
}

impl OperatorEntry {
    pub fn new() -> Self {
        OperatorEntry { prefix: None, suffix: None }
    }
}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Operator {
    pub chars: &'static str,
    pub prec:  u8,
    pub assoc: Assoc,
    pub arity: Arity,
}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Assoc { Left, Right }

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arity { Unary, Binary }

//// HACK: Rust won't derive Clone, PartialEq, or Debug, because of the
//// existential lifetime 'a.  I think this might be a compiler bug.
//// For now, I'll just implement Clone explicitly.
////
////#[derive(Clone, Copy, PartialEq, Eq, Debug)]
//pub enum Arity {
//    Unary  (for<'a> fn(Context, Value<'a>           ) -> Value<'a>),
//    Binary (for<'a> fn(Context, Value<'a>, Value<'a>) -> Value<'a>),
//}
//
//impl Clone for Arity {
//    fn clone(&self) -> Self { *self }
//
//    fn clone_from(&mut self, source: &Self) { *self = *source }
//}
//
//impl Copy for Arity { }
//
//impl PartialEq for Arity {
//    fn eq(&self, other: &Self) -> bool {
//        match (*self, *other) {
//            (Unary (l), Unary (r)) => l as *const () == r as *const (),
//            (Binary(l), Binary(r)) => l as *const () == r as *const (),
//            _ => false,
//        }
//    }
//}
//
//impl Eq for Arity { }
//
//impl Debug for Arity {
//    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
//        match *self {
//            Unary  (m) => write!(f,  "Unary({:p})", m as *const ()),
//            Binary (m) => write!(f, "Binary({:p})", m as *const ()),
//        }
//    }
//}

