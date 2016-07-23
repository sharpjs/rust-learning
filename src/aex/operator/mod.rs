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

