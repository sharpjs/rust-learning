// Interner
//
// This file is part of AEx.
// Copyright (C) 2017 Jeffrey Sharp
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

use aex::mem::arena::Arena;
use aex::mem::ptr::Ptr;

// -----------------------------------------------------------------------------
// Derived Types

pub type StringInterner = Interner<String, str>;

// -----------------------------------------------------------------------------
// Interner

pub struct Interner<T, B: ?Sized = T>
where T: Borrow<B>, B: Hash + Eq {

    // Map from object to its interned object
    map: RefCell<HashMap<Ptr<B>, Ptr<B>>>,

    // The objects owned by this interner
    arena: Arena<T>,

    // SAFETY:
    //
    // Ptr requires that referenced values live at least as long as the Ptr.
    // That is ensured here in two ways:
    //
    // - intern() makes values owned by the arena.  They will not move or drop
    //   during the interner's lifetime.
    //
    // - intern_ref() requires &'static references.
    //
}

impl<T, B: ?Sized> Interner<T, B>
where T: Borrow<B>, B: Hash + Eq {
    pub fn new() -> Self {
        Interner {
            map:   RefCell::new(HashMap::new()),
            arena: Arena::new(),
        }
    }

    pub fn intern(&self, obj: T) -> &B {
        let mut map = self.map.borrow_mut();

        if let Some(&ptr) = map.get(obj.borrow()) {
            return ptr.as_ref()
        }

        let obj = self.arena.alloc(obj) as &T;

        let ptr = Ptr::from(obj.borrow());
        map.insert(ptr, ptr);
        ptr.as_ref()
    }

    pub fn intern_ref(&self, obj: &'static B) -> &B {
        let mut map = self.map.borrow_mut();

        if let Some(&ptr) = map.get(obj) {
            return ptr.as_ref()
        }

        let ptr = Ptr::from(obj);
        map.insert(ptr, ptr);
        ptr.as_ref()
    }
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use super::*;

    // Prefixes guarantee that rustc will emit two separate "Hello" strings.
    const A: &'static str = &"A Hello";
    const B: &'static str = &"B Hello";
    const C: &'static str = &"C olleH";

    #[test]
    fn intern() {
        let a_str = (&A[2..]).to_string();
        let b_str = (&B[2..]).to_string();
        let c_str = (&C[2..]).to_string();

        assert!(a_str.as_ptr() != b_str.as_ptr());

        let interner = StringInterner::new();
        let a_intern = interner.intern(a_str);
        let b_intern = interner.intern(b_str);
        let c_intern = interner.intern(c_str);

        assert!(a_intern.as_ptr() == b_intern.as_ptr());
        assert!(a_intern.as_ptr() != c_intern.as_ptr());
        assert!(b_intern == "Hello");
        assert!(c_intern == "olleH");
    }

    #[test]
    fn intern_ref() {
        let a_ref = &A[2..];
        let b_ref = &B[2..];
        let c_ref = &C[2..];

        assert!(a_ref.as_ptr() != b_ref.as_ptr());

        let interner = StringInterner::new();
        let a_intern = interner.intern_ref(a_ref);
        let b_intern = interner.intern_ref(b_ref);
        let c_intern = interner.intern_ref(c_ref);

        assert!(a_intern.as_ptr() == a_ref.as_ptr());
        assert!(b_intern.as_ptr() == a_ref.as_ptr());
        assert!(c_intern.as_ptr() == c_ref.as_ptr());
    }
}

