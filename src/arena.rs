// Arena Allocator
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

// Inspiration from:
// - https://github.com/SimonSapin/rust-typed-arena
// - https://github.com/rust-lang/rust/blob/1.4.0/src/libarena/lib.rs

use std::cell::RefCell;
use std::mem::{replace, transmute};

const CHUNK_SIZE: usize = 256;

pub struct Arena<T> {
    chunks: RefCell<Chunks<T>>
}

struct Chunks<T> {
    current: Chunk<T>,
    filled:  Vec<Chunk<T>>,
}

struct Chunk<T> (Vec<T>);

impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena {
            chunks: RefCell::new(Chunks {
                current: Chunk::new(),
                filled:  vec![]
            })
        }
    }

    pub fn alloc(&self, object: T) -> &mut T {
        let mut chunks = self.chunks.borrow_mut();

        // Start a new chunk if current one is full
        if chunks.current.is_full() {
            let full = replace(&mut chunks.current, Chunk::new());
            chunks.filled.push(full);
        }

        // Move object into current chunk, and obtain a ref to its new home
        let object = chunks.current.alloc(object);

        // Promote ref lifetime to that of arena
        unsafe { transmute::<&mut T, &mut T>(object) }
    }
}

impl<T> Chunk<T> {
    fn new() -> Self {
        Chunk(Vec::with_capacity(CHUNK_SIZE))
    }

    fn is_full(&self) -> bool {
        self.0.len() == CHUNK_SIZE
    }

    fn alloc(&mut self, object: T) -> &mut T {
        let vec   = &mut self.0;
        let index = vec.len();
        vec.push(object);
        &mut vec[index]
    }
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::char;
    use std::collections::HashSet;
    use super::*;
    use super::CHUNK_SIZE;

    struct Foo<'a> (
        char,                       // id
        &'a RefCell<HashSet<char>>, // ids of dropped Foos
    );

    impl<'a> Drop for Foo<'a> {
        fn drop(&mut self) {
            self.1.borrow_mut().insert(self.0);
        }
    }

    #[test]
    fn test() {
        let dropped = RefCell::new(HashSet::new());

        {
            let arena = Arena::new();
            let a = arena.alloc(Foo('a', &dropped));
            let b = arena.alloc(Foo('b', &dropped));

            // Ensure arena has to grow a new chunk
            for n in 0..CHUNK_SIZE {
                let c = char::from_u32('c' as u32 + n as u32).unwrap();
                arena.alloc(Foo(c, &dropped));
            }

            assert!(a.0 == 'a');
            assert!(b.0 == 'b');
            assert!(dropped.borrow().len() == 0);

            // arena is dropped here
        }

        let dropped = dropped.into_inner();
        assert!(dropped.len() == CHUNK_SIZE + 2);
        assert!(dropped.contains(&'a'));
        assert!(dropped.contains(&'b'));
    }
}

