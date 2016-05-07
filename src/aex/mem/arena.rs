// Arena Allocator
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

// Inspiration from:
// - https://github.com/SimonSapin/rust-typed-arena
// - https://github.com/rust-lang/rust/blob/1.4.0/src/libarena/lib.rs

use std::cell::RefCell;
use std::mem;

use aex::mem::Id;

const CHUNK_SIZE_MIN:     u32 = 0x0000_0001; //   1
const CHUNK_SIZE_DEFAULT: u32 = 0x0000_0100; // 256
const CHUNK_SIZE_MAX:     u32 = 0x0001_0000; // 64K

// -----------------------------------------------------------------------------
// Arena

#[derive(Clone, Debug)]
pub struct Arena<T> {
    chunks: RefCell<Chunks<T>>,
    scale:  Scale,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self::with_chunk_size(CHUNK_SIZE_DEFAULT)
    }

    pub fn with_chunk_size(size: u32) -> Self {
        Arena {
            chunks: RefCell::new(Chunks {
                current: Chunk::new(size as usize),
                filled:  vec![]
            }),
            scale: Scale::new(size),
        }
    }

    pub fn alloc(&self, obj: T) -> Id<T> {
        let mut chunks = self.chunks.borrow_mut();
        let     num    = chunks.ensure(self.scale);
        let     idx    = chunks.current.alloc(obj);

        // Return handle to retrieve the value later
        Id::from(self.scale.pack(num, idx))
    }

    pub fn get(&self, id: Id<T>) -> &T {
        let chunks     = self.chunks.borrow();
        let (num, idx) = self.scale.unpack(u32::from(id));
        let obj        = chunks.get(num).get(idx);

        // Promote ref lifetime to that of &self
        unsafe { mem::transmute::<&T, &T>(obj) }
    }
}

// -----------------------------------------------------------------------------
// Chunks

#[derive(Clone, Debug)]
struct Chunks<T> {
    current: Chunk<T>,
    filled:  Vec<Chunk<T>>,
}

impl<T> Chunks<T> {
    #[inline]
    fn ensure(&mut self, scale: Scale) -> usize {
        let mut num = self.filled.len();

        // Use current chunk if it's not full yet
        if !self.current.is_full() {
            return num
        }

        // Check if arena is growable
        num += 1;
        if num == scale.max_chunks() {
            panic!("Allocation failed. Arena is full.");
        }

        // Grow arena by one chunk
        let size = self.current.size();
        let full = mem::replace(&mut self.current, Chunk::new(size));
        self.filled.push(full);

        num
    }

    #[inline]
    fn get(&self, num: usize) -> &Chunk<T> {
        let count = self.filled.len();

        if      num == count { &self.current     }
        else if num <  count { &self.filled[num] }
        else                 { panic!("Chunk number out of range.") }
    }
}

// -----------------------------------------------------------------------------
// Chunk

#[derive(Clone, Debug)]
struct Chunk<T> (Vec<T>);

impl<T> Chunk<T> {
    #[inline]
    fn new(size: usize) -> Self {
        Chunk(Vec::with_capacity(size))
    }

    #[inline]
    fn size(&self) -> usize {
        self.0.capacity()
    }

    #[inline]
    fn is_full(&self) -> bool {
        self.0.len() == self.0.capacity()
    }

    #[inline]
    fn alloc(&mut self, obj: T) -> usize {
        let idx = self.0.len();
        self.0.push(obj);
        idx
    }

    #[inline]
    fn get(&self, idx: usize) -> &T {
        &self.0[idx]
    }
}

// -----------------------------------------------------------------------------
// Scale

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
struct Scale {
    bits: u32,
    mask: u32,
}

impl Scale {
    fn new(size: u32) -> Self {
        let size =
            if      size < CHUNK_SIZE_MIN { CHUNK_SIZE_MIN }
            else if size > CHUNK_SIZE_MAX { CHUNK_SIZE_MAX }
            else                          { size.next_power_of_two() }
        ;

        let bits = size.trailing_zeros();

        Scale {
            bits: bits,
            mask: (1 << bits) - 1,
        }
    }

    #[inline]
    fn max_chunks(&self) -> usize {
        (1 as usize) << (32 - self.bits)
    }

    #[inline]
    fn pack(&self, num: usize, idx: usize) -> u32 {
        (num as u32) << self.bits |
        (idx as u32)  & self.mask
    }

    #[inline]
    fn unpack(&self, n: u32) -> (usize, usize) {(
        (n >> self.bits) as usize, // num
        (n  & self.mask) as usize, // idx
    )}
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::char;
    use std::collections::HashSet;
    use std::marker::PhantomData;
    use super::*;
    use aex::mem::Id;

    const CHUNK_SIZE: u32 = 8;

    #[derive(Clone, Debug)]
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
    fn alloc_1() {
        let arena = Arena::new();

        let a = arena.alloc('a');

        assert_eq!(a, Id(0, PhantomData));
        assert_eq!(arena.get(a), &'a');
    }

    #[test]
    fn alloc_2() {
        let arena = Arena::new();

        let a = arena.alloc('a');
        let b = arena.alloc('b');

        assert_eq!(a, Id(0, PhantomData));
        assert_eq!(b, Id(1, PhantomData));
        assert_eq!(arena.get(a), &'a');
        assert_eq!(arena.get(b), &'b');
    }

    #[test]
    fn alloc_expand() {
        let arena = Arena::with_chunk_size(CHUNK_SIZE);

        // Ensure arena has to grow a new chunk
        fill_chunk(&arena, |c| c);

        let a = arena.alloc('a');
        let b = arena.alloc('b');

        assert_eq!(a, Id(CHUNK_SIZE as u32 + 0, PhantomData));
        assert_eq!(b, Id(CHUNK_SIZE as u32 + 1, PhantomData));
        assert_eq!(arena.get(a), &'a');
        assert_eq!(arena.get(b), &'b');
    }

    #[test]
    fn drop() {
        let dropped = RefCell::new(HashSet::new());

        {
            let arena = Arena::with_chunk_size(CHUNK_SIZE);
            let a = arena.alloc(Foo('a', &dropped));
            let b = arena.alloc(Foo('b', &dropped));

            // Ensure arena has to grow a new chunk
            fill_chunk(&arena, |c| Foo(c, &dropped));

            assert_eq!(arena.get(a).0, 'a');
            assert_eq!(arena.get(b).0, 'b');
            assert_eq!(dropped.borrow().len(), 0);

            // arena is dropped here
        }

        let dropped = dropped.into_inner();
        assert_eq!(dropped.len(), (CHUNK_SIZE + 2) as usize);
        assert!(dropped.contains(&'a'));
        assert!(dropped.contains(&'b'));
    }

    fn fill_chunk<T, F>(arena: &Arena<T>, f: F)
                       where F: Fn(char) -> T {
        for n in 0..CHUNK_SIZE {
            let c = 'c' as u32 + n;
            let c = char::from_u32(c).unwrap();
            arena.alloc(f(c));
        }
    }
}

