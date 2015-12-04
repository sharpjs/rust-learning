// Identifier Scope
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

use std::collections::HashMap;

use arena::*;
use symbol::*;
use types::*;

#[derive(Clone, Debug)]
pub struct Symbol<'a> {
    pub name: &'a str,
    pub ty:   &'a Type<'a>,
}

pub struct Scope<'a> {
    parent:  Option<&'a Scope<'a>>,
    symbols: HashMap<&'a str, Symbol<'a>>,
    types:   HashMap<&'a str, Type<'a>>,
}

pub struct ScopeMap<'a, T: 'a> {
    map:   HashMap<&'a str, &'a T>,
    arena: Arena<T>,
}

impl<'a> Scope<'a> {
    fn new(parent: Option<&'a Self>) -> Self {
        Scope {
            parent:  parent,
            symbols: HashMap::new(),
            types:   HashMap::new()
        }
    }

    pub fn new_root() -> Self {
        Self::new(None)
    }

    pub fn new_subscope(&'a self) -> Self {
        Self::new(Some(self))
    }

    pub fn define_symbol(&mut self, sym: Symbol<'a>) -> Result<(), Symbol<'a>> {
        match self.symbols.insert(sym.name, sym) {
            None    => Ok(()),
            Some(s) => Err(s)
        }
    }

    pub fn lookup_symbol(&'a self, name: &str) -> Option<&'a Symbol<'a>> {
        self.symbols.get(name)
    }

    pub fn define_type(&mut self, name: &'a str, ty: Type<'a>) -> Result<(), Type<'a>> {
        match self.types.insert(name, ty) {
            None    => Ok(()),
            Some(t) => Err(t)
        }
    }

    pub fn lookup_type(&'a self, name: &str) -> Option<&'a Type<'a>> {
        self.types.get(name)
    }
}

impl<'a, T: 'a> ScopeMap<'a, T> {
    fn new() -> Self {
        ScopeMap { map: HashMap::new(), arena: Arena::new() }
    }

    fn add(&'a mut self, object: T) -> &'a T {
        self.arena.alloc(object)
    }

    fn define(&'a mut self, name: &'a str, object: T) -> Result<(), &'a T> {
        let object = self.arena.alloc(object);

        match self.map.insert(name, object) {
            None    => Ok(()),
            Some(o) => Err(o)
        }
    }

    fn define_ref(&'a mut self, name: &'a str, object: &'a T) -> Result<(), &'a T> {
        match self.map.insert(name, object) {
            None    => Ok(()),
            Some(o) => Err(o)
        }
    }

    fn lookup(&self, name: &str) -> Option<&T> {
        self.map.get(&name).map(|x| *x)
    }
}

