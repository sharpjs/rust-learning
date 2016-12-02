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

#[macro_use]
pub mod dispatch;
pub mod builtin;

use std::borrow::Borrow;
use std::collections::HashMap;

use self::Assoc::*;

pub use self::dispatch::{UnaryOperator, BinaryOperator};

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Operator {
    pub chars: &'static str,    // characters
    pub prec:  u8,              // precedence level
    pub assoc: Assoc,           // associativity
}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Assoc { Left, Right }

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arity { Unary, Binary }

// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OperatorTable {
    map: HashMap<&'static str, OperatorEntry>,
}

impl OperatorTable {
    pub fn new() -> Self {
        OperatorTable { map: HashMap::new() }
    }

    pub fn add<O: AnOperator>(&mut self, operator: &'static O) {
        let entry = self.map
            .entry(operator.base().chars)
            .or_insert_with(|| OperatorEntry::new());

        operator.add_to(entry)
    }

    pub fn map(&self) -> &HashMap<&'static str, OperatorEntry> {
        &self.map
    }

    pub fn get_prefix<S: Borrow<str>>(&self, chars: S)
                     -> Option<&'static UnaryOperator> {
        self.map
            .get(chars.borrow())
            .and_then(|e| e.prefix)
    }

    pub fn get_suffix<S: Borrow<str>>(&self, chars: S)
                     -> Option<AnyOperator> {
        self.map
            .get(chars.borrow())
            .and_then(|e| e.suffix)
    }
}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct OperatorEntry {
    pub prefix: Option<&'static UnaryOperator>, // prefix
    pub suffix: Option<           AnyOperator>, // infix or postfix
}

impl OperatorEntry {
    pub fn new() -> Self {
        OperatorEntry { prefix: None, suffix: None }
    }
}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AnyOperator {
    Unary  (&'static UnaryOperator),
    Binary (&'static BinaryOperator),
}

// -----------------------------------------------------------------------------

pub trait AnOperator {
    fn base(&self) -> &Operator;

    fn arity(&self) -> Arity;

    fn add_to(&'static self, entry: &mut OperatorEntry);
}

impl AnOperator for UnaryOperator {
    fn base(&self) -> &Operator { &self.base }

    fn arity(&self) -> Arity { Arity::Unary }

    fn add_to(&'static self, entry: &mut OperatorEntry) {
        match self.base.assoc {
            Right => entry.prefix = Some(self),
            Left  => entry.suffix = Some(AnyOperator::Unary(self)),
        }
    }
}

impl AnOperator for BinaryOperator {
    fn base(&self) -> &Operator { &self.base }

    fn arity(&self) -> Arity { Arity::Binary }

    fn add_to(&'static self, entry: &mut OperatorEntry) {
        entry.suffix = Some(AnyOperator::Binary(self))
    }
}

