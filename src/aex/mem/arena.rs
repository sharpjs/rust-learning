// Reference Pool
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

use aex::mem::{Id, promote_lifetime};

// -----------------------------------------------------------------------------
// RefPool

const DEFAULT_CAPACITY: usize = 256;

#[derive(Clone, Debug)]
pub struct RefPool<T> (RefCell<Vec<T>>);

impl<T> RefPool<T> {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        RefPool(RefCell::new(Vec::with_capacity(capacity)))
    }

    pub fn capacity(&self) -> usize {
        self.0.borrow().capacity()
    }

    pub fn alloc(&self, obj: T) -> Id<T> {
        let mut vec = self.0.borrow_mut();
        let     idx = vec.len();
        vec.push(obj);
        Id::from(idx)
    }

    pub fn get<R>(&self, id: Id<T>) -> &R where T: AsRef<R> {
        let vec = self.0.borrow();
        let idx = usize::from(id);
        let obj = vec[idx].as_ref();
        unsafe { promote_lifetime(obj) }
    }
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    //use std::cell::RefCell;
    //use std::char;
    //use std::collections::HashSet;
    //use std::marker::PhantomData;
    use super::*;
    use aex::mem::Id;

    //const CHUNK_SIZE: u32 = 8;

    //#[derive(Clone, Debug)]
    //struct Foo<'a> (
    //    char,                       // id
    //    &'a RefCell<HashSet<char>>, // ids of dropped Foos
    //);

    //impl<'a> Drop for Foo<'a> {
    //    fn drop(&mut self) {
    //        self.1.borrow_mut().insert(self.0);
    //    }
    //}

    #[test]
    fn alloc_1() {
        let pool = RefPool::new();

        let a = pool.alloc(Box::new('a'));

        assert_eq!(a,           Id::from(0));
        assert_eq!(pool.get(a), &'a');
    }

    //#[test]
    //fn alloc_2() {
    //    let pool = RefPool::new();

    //    let a = pool.alloc('a');
    //    let b = pool.alloc('b');

    //    assert_eq!(a, Id(0, PhantomData));
    //    assert_eq!(b, Id(1, PhantomData));
    //    assert_eq!(pool.get(a), &'a');
    //    assert_eq!(pool.get(b), &'b');
    //}

    //#[test]
    //fn alloc_expand() {
    //    let pool = RefPool::with_chunk_size(CHUNK_SIZE);

    //    // Ensure pool has to grow a new chunk
    //    fill_chunk(&pool, |c| c);

    //    let a = pool.alloc('a');
    //    let b = pool.alloc('b');

    //    assert_eq!(a, Id(CHUNK_SIZE as u32 + 0, PhantomData));
    //    assert_eq!(b, Id(CHUNK_SIZE as u32 + 1, PhantomData));
    //    assert_eq!(pool.get(a), &'a');
    //    assert_eq!(pool.get(b), &'b');
    //}

    //#[test]
    //fn drop() {
    //    let dropped = RefCell::new(HashSet::new());

    //    {
    //        let pool = RefPool::with_chunk_size(CHUNK_SIZE);
    //        let a = pool.alloc(Foo('a', &dropped));
    //        let b = pool.alloc(Foo('b', &dropped));

    //        // Ensure pool has to grow a new chunk
    //        fill_chunk(&pool, |c| Foo(c, &dropped));

    //        assert_eq!(pool.get(a).0, 'a');
    //        assert_eq!(pool.get(b).0, 'b');
    //        assert_eq!(dropped.borrow().len(), 0);

    //        // pool is dropped here
    //    }

    //    let dropped = dropped.into_inner();
    //    assert_eq!(dropped.len(), (CHUNK_SIZE + 2) as usize);
    //    assert!(dropped.contains(&'a'));
    //    assert!(dropped.contains(&'b'));
    //}

    //fn fill_chunk<T, F>(pool: &RefPool<T>, f: F)
    //                   where F: Fn(char) -> T {
    //    for n in 0..CHUNK_SIZE {
    //        let c = 'c' as u32 + n;
    //        let c = char::from_u32(c).unwrap();
    //        pool.alloc(f(c));
    //    }
    //}
}

