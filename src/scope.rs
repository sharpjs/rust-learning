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
use std::rc::Rc;

use types::*;
use util::shared::*;

pub type SharedSymbol = Rc<Symbol>;

#[derive(Clone, Debug)]
pub struct Symbol {
    pub name: SharedStr,
    pub ty:   SharedType,
}

pub struct Scope<'p> {
    parent:  Option<&'p Scope<'p>>,
    symbols: HashMap<SharedStr, SharedSymbol>,
    types:   HashMap<SharedStr, SharedType>,
}

impl<'p> Scope<'p> {
    fn new(parent: Option<&'p Self>) -> Self {
        Scope {
            parent:  parent,
            symbols: HashMap::new(),
            types:   HashMap::new()
        }
    }

    pub fn new_root() -> Self {
        Self::new(None)
    }

    pub fn new_subscope<'s>(&'s self) -> Scope<'s> {
        Self::new(Some(self))
    }

    pub fn define_symbol(&mut self, sym: SharedSymbol)
                         -> Result<(), SharedSymbol> {
        let name  = sym.name.clone();
        let prior = self.symbols.insert(name, sym);
        match prior {
            None    => Ok(()),
            Some(s) => Err(s)
        }
    }

    pub fn lookup_symbol(&self, name: &str) -> Option<SharedSymbol> {
        self.symbols.get(name).cloned()
    }

    pub fn define_type(&mut self, name: SharedStr, ty: SharedType)
                       -> Result<(), SharedType> {
        let prior = self.types.insert(name, ty);
        match prior {
            None    => Ok(()),
            Some(t) => Err(t)
        }
    }

    pub fn lookup_type(&self, name: &str) -> Option<SharedType> {
        self.types.get(name).cloned()
    }
}

