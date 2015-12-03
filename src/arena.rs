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
    use super::*;

    struct Foo(usize);

    #[test]
    fn test() {
        let arena = Arena::<Foo>::new();

        // TODO
    }
}

