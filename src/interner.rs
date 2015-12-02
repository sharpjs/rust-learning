// String Interner
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

use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

// -----------------------------------------------------------------------------
// Handle for Interned Object

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StrId(usize);

impl From<usize> for StrId {
    #[inline]
    fn from(n: usize) -> Self { StrId(n) }
}

impl Into<usize> for StrId {
    #[inline]
    fn into(self) -> usize { self.0 }
}

impl fmt::Debug for StrId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

// -----------------------------------------------------------------------------
// Interned Object

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct Interned<T> (Rc<T>);

impl<T> Interned<T> {
    #[inline]
    fn new<S: Into<T>>(s: S) -> Self {
        Interned(Rc::new(s.into()))
    }
}

// Necessary to enable &str to probe HashMap keyed by Rc<string>
impl Borrow<str> for Interned<String> {
    #[inline]
    fn borrow(&self) -> &str {
        &self.0[..]
    }
}

// -----------------------------------------------------------------------------
// Interner

#[derive(Clone)]
pub struct Interner {
    map: RefCell<HashMap<Interned<String>, usize>>,
    vec: RefCell<Vec    <Interned<String>       >>,
}

impl Interner {
    pub fn new() -> Self {
        Interner {
            map: RefCell::new(HashMap::new()),
            vec: RefCell::new(Vec    ::new()),
        }
    }

    pub fn intern<S: AsRef<str>>(&self, val: S) -> StrId {
        // If an interned copy exists, return it
        let mut map = self.map.borrow_mut();
        let     val = val.as_ref();

        if let Some(&idx) = map.get(val) {
            return idx.into();
        }

        // Else, intern a copy
        let mut vec = self.vec.borrow_mut();
        let     idx = vec.len();
        let     val = Interned::new(val);

        vec.push(val.clone());
        map.insert(val, idx);
        idx.into()
    }

    pub fn add<S: AsRef<str>>(&self, val: S) -> StrId {
        let mut vec = self.vec.borrow_mut();
        let     idx = vec.len();
        let     val = Interned::new(val.as_ref());

        vec.push(val);
        idx.into()
    }

    #[inline]
    pub fn get(&self, id: StrId) -> Rc<String> {
        self.vec.borrow()[id.0].0.clone()
    }
}

// WIP towards an arena-based interner
use std::hash::Hash;
use arena::*;

pub struct Interner2<'a, T, B=T> where T: Borrow<B>, B: 'a + Hash + Eq {
    map:   RefCell<HashMap<&'a B, &'a B>>,
    arena: Arena<T>,
}

impl<'a, T, B> Interner2<'a, T, B> where T: Borrow<B>, B: 'a + Hash + Eq {
    pub fn intern(&'a self, object: T) -> &'a B {
        let mut map = self.map.borrow_mut();

        if let Some(&object) = map.get(&object.borrow()) {
            return object;
        }

        let object = self.arena.alloc(object) as &T;
        let object = object.borrow();
        map.insert(object, object);
        object
    }
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern() {
        let t = Interner::new();

        let a = t.intern("Hello");
        let b = t.intern(&"Hello".to_string());

        assert!   (a == b);
        assert_eq!("Hello", *t.get(a));
    }

    #[test]
    fn add() {
        let t = Interner::new();

        let a = t.add("Hello");
        let b = t.add(&"Hello".to_string());

        assert!   (a != b);
        assert_eq!("Hello", *t.get(a));
        assert_eq!("Hello", *t.get(b));
    }
}

