// Interner
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

use std::borrow::{Borrow, Cow};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Index;

use aex::mem::Id;

// -----------------------------------------------------------------------------

pub type Strings<'a> = Interner<'a, str>;

// -----------------------------------------------------------------------------

pub struct Interner<'a, B: 'a + ToOwned + Hash + Eq + ?Sized> {
    // Map from objects to identifiers
    map: RefCell<HashMap<Cow<'a, B>, usize>>,

    // Map from identifiers to objects
    vec: RefCell<Vec<*const B>>,
}

const DEFAULT_CAPACITY: usize = 256;

impl<'a, B: 'a + ToOwned + Hash + Eq + ?Sized> Interner<'a, B> {
    #[inline(always)]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Interner {
            map: RefCell::new(HashMap::with_capacity(capacity)),
            vec: RefCell::new(Vec    ::with_capacity(capacity)),
        }
    }

    pub fn capacity(&self) -> usize {
        self.vec.borrow().capacity()
    }

    #[inline(always)]
    pub fn intern(&self, obj: B::Owned) -> Id<B> {
        self.intern_cow(Cow::Owned(obj))
    }

    #[inline(always)]
    pub fn intern_ref(&self, obj: &'a B) -> Id<B> {
        self.intern_cow(Cow::Borrowed(obj))
    }

    fn intern_cow(&self, obj: Cow<'a, B>) -> Id<B> {
        let mut map = self.map.borrow_mut();

        if let Some(&idx) = map.get(&obj) {
            return Id::from(idx);
        }

        let mut vec = self.vec.borrow_mut();
        let     idx = vec.len();

        // SAFETY: While obj will move, the address it points to will not.
        vec.push(obj.as_ref() as *const _);
        map.insert(obj, idx);
        Id::from(idx)
    }

    pub fn get(&self, id: Id<B>) -> &B {
        let idx = usize::from(id);
        let ptr = self.vec.borrow()[idx];

        // SAFETY: obj lives at least as long as self.
        unsafe { &*ptr }
    }
}

impl<'a, B: 'a + ToOwned + Hash + Eq + ?Sized>
Index<Id<B>> for Interner<'a, B> {
    type Output = B;

    #[inline(always)]
    fn index(&self, id: Id<B>) -> &B { self.get(id) }
}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern_equal() {
        let a_val = "Hello".to_string();
        let b_val = "Hello".to_string();

        // Verify separate Strings with same chars
        assert!(&a_val as *const _ != &b_val as *const _);

        let i    = Strings::new();
        let a_id = i.intern(a_val);
        let b_id = i.intern(b_val);

        assert!(a_id == b_id);
        assert_eq!(i.get(a_id), "Hello");
        assert_eq!(i.get(b_id), "Hello");

        assert!(i.get(a_id) as *const _ == i.get(b_id) as *const _);
    }

    #[test]
    fn intern_ref_equal() {
        let a_val = "Hello".to_string();
        let b_val = "Hello".to_string();
        let a_ref = a_val.as_ref();
        let b_ref = b_val.as_ref();

        // Verify separate Strings with same chars
        assert!(a_ref as *const _ != b_ref as *const _);

        let i    = Strings::new();
        let a_id = i.intern_ref(a_ref);
        let b_id = i.intern_ref(b_ref);

        assert!(a_id == b_id);

        assert_eq!(i.get(a_id), "Hello");
        assert_eq!(i.get(b_id), "Hello");

        assert!(i.get(a_id) as *const _ == a_ref as *const _);
        assert!(i.get(b_id) as *const _ == a_ref as *const _); // NOT b_ref
    }

    #[test]
    fn intern_diff() {
        let a_val = "Hello".to_string();
        let b_val = "elloH".to_string();

        let i    = Strings::with_capacity(2);
        let a_id = i.intern(a_val);
        let b_id = i.intern(b_val);

        assert!(a_id != b_id);
        assert_eq!(i.get(a_id), "Hello");
        assert_eq!(i.get(b_id), "elloH");
    }

    #[test]
    fn intern_ref_diff() {
        let a_val = "Hello".to_string();
        let b_val = "elloH".to_string();

        let a_ref = a_val.as_ref();
        let b_ref = b_val.as_ref();

        let i    = Strings::with_capacity(2);
        let a_id = i.intern_ref(a_ref);
        let b_id = i.intern_ref(b_ref);

        assert!(a_id != b_id);

        assert_eq!(i.get(a_id), "Hello");
        assert_eq!(i.get(b_id), "elloH");

        assert!(i.get(a_id) as *const _ == a_ref as *const _);
        assert!(i.get(b_id) as *const _ == b_ref as *const _);
    }

    #[test]
    fn no_move() {
        let i = Strings::with_capacity(2);

        let a_id = i.intern("a".to_string());

        assert!(i.capacity() == 2);
        let a_p0 = i.get(a_id) as *const _;

        i.intern("b".to_string());
        i.intern("c".to_string());
        i.intern("d".to_string());
        i.intern("e".to_string());
        i.intern("f".to_string());

        assert!(i.capacity() != 2);
        let a_p1 = i.get(a_id) as *const _;

        // When the internal vector grows, the interned values do not move.
        assert!(a_p0 == a_p1);
    }

    #[test]
    #[should_panic]
    fn id_out_of_range() {
        use aex::mem::Id;

        let i = Strings::with_capacity(2);
        i.get(Id::from(42));
    }
}

