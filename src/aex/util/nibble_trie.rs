// Trie Using N-Bit Key Chunks
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

use std::marker::PhantomData;
//use std::ops::{BitAnd, Shr};

// This is just an experiment.  Don't know if this will stay.

pub struct NibbleTrie<'a, K, T>
where
    K: Into<usize>,
    T: 'a + ?Sized
{
    // Flat list of nodes in the trie.  Node 0 is the root.
    nodes: Vec<Node<'a, T>>,

    m: PhantomData<K>,
}

struct Node<'a, T: 'a + ?Sized> {
    // Leaf part
    value: Option<&'a T>,  // Value if no more key

    // Non-leaf part
    pos:   u8,          // bit position of key subset
    len:   u8,          // bit length   of key subset
    mask:  u16,         // mask         of key subset (1 = significant)
    nodes: Vec<u16>,    // indexes of nodes identified by key subset
}

impl<'a, K, T> NibbleTrie<'a, K, T>
where
    K: Into<usize>,
    T: 'a + ?Sized
{
    pub fn from<I>(src: &mut I) -> Self
    where I: Iterator<Item=(u32, u32, &'a T)>{
        Self { nodes: vec![], m: PhantomData }
    }
}

