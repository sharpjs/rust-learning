// Scopes
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

use aex::mem::arena::*;
use aex::symbol::*;
use aex::types::*;

pub struct Scope<'a> {
    pub symbols: ScopeMap<'a, Symbol<'a>>,
    pub types:   ScopeMap<'a, Type  <'a>>,
}

pub struct ScopeMap<'a, T: 'a> {
    map:    HashMap<&'a str, &'a T>,
    arena:  Arena<T>,
    parent: Option<&'a ScopeMap<'a, T>>,
}

impl<'a> Scope<'a> {
    fn new(parent: Option<&'a Self>) -> Self {
        Scope {
            symbols: ScopeMap::new(parent.map(|p| &p.symbols)),
            types:   ScopeMap::new(parent.map(|p| &p.types  )),
        }
    }

    pub fn new_root() -> Self {
        Self::new(None)
    }

    pub fn new_subscope(&'a self) -> Self {
        Self::new(Some(self))
    }
}

impl<'a, T: 'a> ScopeMap<'a, T> {
    fn new(parent: Option<&'a ScopeMap<'a, T>>) -> Self {
        ScopeMap {
            map:    HashMap::new(),
            arena:  Arena::new(),
            parent: parent,
        }
    }

    pub fn add(&mut self, object: T) -> &T {
        self.add_internal(object)
    }

    pub fn define(&mut self, name: &'a str, object: T)
                 -> Result<(), &T> {
        let object = self.add_internal(object);

        match self.map.insert(name, object) {
            None    => Ok(()),
            Some(o) => Err(o)
        }
    }

    pub fn define_ref(&mut self, name: &'a str, object: &'a T)
                     -> Result<(), &T> {
        match self.map.insert(name, object) {
            None    => Ok(()),
            Some(o) => Err(o)
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&T> {
        self.lookup_internal(name)
    }

    #[inline]
    fn add_internal(&mut self, object: T) -> &'a T {
        use std::mem::transmute;

        // SAFETY: We move the object into the arena and receive a borrow to it.
        //   We then must use transmute to promote the borrow's lifetime to that
        //   required by `map`.  The new lifetime ('a) might exceed the arena's
        //   lifetime.  That is OK, because the lifetime is reconstrained to
        //   that of &self by the exposed functions of this type, and because
        //   the arena will not drop the object within the lifetime of &self.
        //   
        let object: &   T = self.arena.alloc(object);
        let object: &'a T = unsafe { transmute(object) };
        object
    }

    fn lookup_internal(&self, name: &str) -> Option<&'a T> {
        // SAFETY: See note above.  This method must not be public, because all
        //   exposed methods of this type must constrain the lifetime of the
        //   return value to that of &self.

        // First, look in this map
        if let Some(&object) = self.map.get(&name) {
            return Some(object);
        }

        // Else look in parent scope, if any
        if let Some(parent) = self.parent {
            return parent.lookup_internal(name);
        }

        // Else fail
        None
    }
}

#[cfg(test)]
mod tests {

    mod scope {
        use super::super::*;

        #[test]
        fn new() {
            let parent = Scope::new_root();
            let child  = parent.new_subscope();

            assert_eq!(
                child.types.parent.unwrap() as *const _,
                &parent.types               as *const _
            );
            assert_eq!(
                child.symbols.parent.unwrap() as *const _,
                &parent.symbols               as *const _
            );
        }
    }

    mod scope_map {
        use super::super::*;
        use aex::types::*;

        #[test]
        fn empty() {
            let map = ScopeMap::<Type>::new(None);

            assert_eq!( map.lookup("any"), None );
            assert_eq!( map.lookup("any"), None );
        }

        #[test]
        fn defined() {
            let mut map = ScopeMap::new(None);

            assert_eq!( map.define("t", U32.clone()), Ok  (( )) );
            assert_eq!( map.lookup("t"),              Some(U32) );
        }

        #[test]
        fn inherited() {
            let mut parent = ScopeMap::new(None);

            assert_eq!( parent.define("t", U32.clone()), Ok(()) );

            let map = ScopeMap::new(Some(&parent));

            assert_eq!( map.lookup("t"), Some(U32) );
        }

        #[test]
        fn overridden() {
            let mut parent = ScopeMap::new(None);

            assert_eq!( parent.define("t", U16.clone()), Ok(()) );

            let mut map = ScopeMap::new(Some(&parent));

            assert_eq!( map.define("t", U32.clone()), Ok  (( )) );
            assert_eq!( map.lookup("t"),              Some(U32) );
        }

        #[test]
        fn duplicate() {
            let mut map = ScopeMap::new(None);

            assert_eq!( map.define("t", U16.clone()), Ok  (( )) );
            assert_eq!( map.define("t", U32.clone()), Err (U16) );
            assert_eq!( map.lookup("t"),              Some(U32) );
        }
    }
}

