// Scopes
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

use std::collections::HashMap;

use aex::mem::Arena;
use aex::symbol::Symbol;
use aex::types::res::ResolvedType;
use aex::util::Lookup;

// -----------------------------------------------------------------------------

pub trait Scoped<'a> {
    fn symbols (&self) -> &Lookup<str, Symbol      <'a>>;
    fn types   (&self) -> &Lookup<str, ResolvedType<'a>>;
}

// -----------------------------------------------------------------------------

pub struct Scope<'a> {
    pub symbols: SymbolScope<'a>,
    pub types:   TypeScope  <'a>,
}

pub type SymbolScope<'a> = ScopeMap<'a, Symbol      <'a>>;
pub type TypeScope  <'a> = ScopeMap<'a, ResolvedType<'a>>;

impl<'a> Scope<'a> {
    pub fn new() -> Self {
        Scope {
            symbols: ScopeMap::new(None),
            types:   ScopeMap::new(None),
        }
    }

    pub fn with_parent<'p: 'a>(parent: &'a Scoped<'p>) -> Self {
        use std::mem::transmute;

        // SAFETY:  Scoped<'p> is invariant in 'p.  It is OK to transmute the
        // parent reference to Scoped<'a> here, because 'p >= 'a, and because
        // this object never mutates its parent.
        //
        // Rustonomicon: Subtyping and Variance
        // https://doc.rust-lang.org/stable/nomicon/subtyping.html
        //
        let parent: &'a Scoped<'a> = unsafe { transmute(parent) };

        Scope {
            symbols: ScopeMap::new(Some(parent.symbols())),
            types:   ScopeMap::new(Some(parent.types  ())),
        }
    }
}

impl<'a> Scoped<'a> for Scope<'a> {
    #[inline]
    fn symbols(&self) -> &Lookup<str, Symbol<'a>> {
        &self.symbols
    }

    #[inline]
    fn types(&self) -> &Lookup<str, ResolvedType<'a>> {
        &self.types
    }
}

impl<'a> Lookup<str, Symbol<'a>> for Scope<'a> {
    #[inline]
    fn lookup(&self, name: &str) -> Option<&Symbol<'a>> {
        self.symbols.lookup(name)
    }
}

impl<'a> Lookup<str, ResolvedType<'a>> for Scope<'a> {
    #[inline]
    fn lookup(&self, name: &str) -> Option<&ResolvedType<'a>> {
        self.types.lookup(name)
    }
}

// -----------------------------------------------------------------------------

pub struct ScopeMap<'a, T: 'a> {
    map:    HashMap<&'a str, &'a T>,
    arena:  Arena<T>,
    parent: Option<&'a Lookup<str, T>>,
}

impl<'a, T: 'a> ScopeMap<'a, T> {
    fn new(parent: Option<&'a Lookup<str, T>>) -> Self {
        ScopeMap {
            map:    HashMap::new(),
            arena:  Arena::new(),
            parent: parent,
        }
    }

    pub fn define(&mut self, name: &'a str, obj: T) -> Result<(), &'a T> {
        // SAFETY:  We move obj into the arena and receive a borrow to it.
        // We then promote the borrow's lifetime to 'a, as required by the map.
        // The new lifetime 'a might exceed the arena's lifetime.  That is OK,
        // because lookup() constrains its result to the lifetime of &self.
        // Also, the arena will not drop obj within/ the lifetime of &self.
        //   
        let obj: *const T = self.arena.alloc(obj);
        let obj: &'a    T = unsafe { &*obj };

        self.define_ref(name, obj)
    }

    pub fn define_ref(&mut self, name: &'a str, obj: &'a T) -> Result<(), &'a T> {
        match self.map.insert(name, obj) {
            None      => Ok(()),
            Some(obj) => Err(obj)
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&'a T> {
        // First, look in this map
        if let Some(&obj) = self.map.get(name) {
            return Some(obj);
        }

        // Else, look in parent scope, if any
        if let Some(parent) = self.parent {
            return parent.lookup(name);
        }

        // Else, fail
        None
    }
}

impl<'a, T> Lookup<str, T> for ScopeMap<'a, T> {
    #[inline(always)]
    fn lookup(&self, name: &str) -> Option<&T> {
        self.lookup(name)
    }
}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    mod scope {
        use super::super::*;
        use aex::util::ref_eq;
        use aex::target::{TargetRef, TestTarget};

        #[test]
        fn new() {
            let parent = Scope::new();
            let child  = Scope::with_parent(&parent);

            assert!(ref_eq(child.types  .parent.unwrap(), &parent.types  ));
            assert!(ref_eq(child.symbols.parent.unwrap(), &parent.symbols));
        }

        #[test]
        fn abstract_parent() {
            let target = TestTarget::new();
            let parent = TargetRef::new(&target);
            let child  = Scope::with_parent(&parent);
            // Just making sure it typechecks for now
        }
    }

    mod scope_map {
        use super::super::*;
        use aex::types::Type;
        use aex::types::builtin::{U16, U32};

        #[test]
        fn empty() {
            let map = ScopeMap::<Type>::new(None);

            assert_eq!( map.lookup("any"), None );
            assert_eq!( map.lookup("any"), None );
        }

        #[test]
        fn defined_own() {
            let mut map = ScopeMap::new(None);

            assert_eq!( map.define("t", U32.clone()), Ok(()) );

            assert_eq!( map.lookup("t"), Some(&U32) );
        }

        #[test]
        fn defined_ref() {
            let mut map = ScopeMap::new(None);

            assert_eq!( map.define_ref("t", &U32), Ok(()) );

            assert_eq!( map.lookup("t"), Some(&U32) );
        }

        #[test]
        fn inherited() {
            let mut parent = ScopeMap::new(None);

            assert_eq!( parent.define_ref("t", &U32), Ok(()) );

            let map = ScopeMap::new(Some(&parent));

            assert_eq!( map.lookup("t"), Some(&U32) );
        }

        #[test]
        fn overridden() {
            let mut parent = ScopeMap::new(None);

            assert_eq!( parent.define_ref("t", &U16), Ok(()) );

            let mut map = ScopeMap::new(Some(&parent));

            assert_eq!( map.define_ref("t", &U32), Ok(()) );

            assert_eq!( map.lookup("t"), Some(&U32) );
        }

        #[test]
        fn duplicate() {
            let mut map = ScopeMap::new(None);

            assert_eq!( map.define_ref("t", &U16), Ok(()) );

            assert_eq!( map.define_ref("t", &U32), Err(&U16) );

            assert_eq!( map.lookup("t"), Some(&U32) );
        }
    }
}

