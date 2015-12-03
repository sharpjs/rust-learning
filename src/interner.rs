// Interner
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
use std::hash::Hash;
use arena::*;

pub type StringInterner<'a> = Interner<'a, String, str>;

pub struct Interner<'a, T, B: ?Sized = T>
where T: Borrow<B>,
      B: 'a + Hash + Eq {

    // Map from object to its interned object
    map: RefCell<HashMap<&'a B, &'a B>>,

    // The objects owned by this interner
    arena: Arena<T>,
}

impl<'a, T, B: ?Sized> Interner<'a, T, B>
where T: Borrow<B>,
      B: 'a + Hash + Eq {

    pub fn new() -> Self {
        Interner {
            map:   RefCell::new(HashMap::new()),
            arena: Arena::new(),
        }
    }

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

    pub fn intern_ref(&'a self, object: &'a B) -> &'a B {
        let mut map = self.map.borrow_mut();

        if let Some(&object) = map.get(&object) {
            return object;
        }

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
        let a_str = "Hello".to_string();
        let b_str = "Hello".to_string();
        let c_str = "olleH".to_string();

        assert!(&a_str as *const String != &b_str as *const String);

        let interner = StringInterner::new();
        let a_intern = interner.intern(a_str);
        let b_intern = interner.intern(b_str);
        let c_intern = interner.intern(c_str);

        assert!(a_intern as *const str == b_intern as *const str);
        assert!(a_intern as *const str != c_intern as *const str);
        assert!(b_intern == "Hello");
        assert!(c_intern == "olleH");
    }

    #[test]
    fn intern_ref() {
        let a_str = "Hello".to_string();
        let b_str = "Hello".to_string();
        let c_str = "olleH".to_string();

        let a_ref = a_str.as_ref();
        let b_ref = b_str.as_ref();
        let c_ref = c_str.as_ref();

        assert!(a_ref as *const str != b_ref as *const str);

        let interner = StringInterner::new();
        let a_intern = interner.intern_ref(a_ref);
        let b_intern = interner.intern_ref(b_ref);
        let c_intern = interner.intern_ref(c_ref);

        assert!(a_intern as *const str == a_ref as *const str);
        assert!(b_intern as *const str == a_ref as *const str);
        assert!(c_intern as *const str == c_ref as *const str);
    }
}

