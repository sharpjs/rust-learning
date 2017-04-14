
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

// This is just an experiment.  Don't know if this will stay.
// In fact, I think it will go.

/*

use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{BitAndAssign, BitOrAssign, Not}
//use std::ops::{BitAnd, Shr};

pub struct DecodeTrie<'a, K, T>
where
    K: Into<usize>,
    T: 'a
{
    /// The items mapped by this trie.
    items: &'a [T],

    /// Trie nodes. First item is root node.
    nodes: Vec<Node>,

    k: PhantomData<K>,
}

struct Node {
    /// Index of mapped item if end-of-key reached
    item_idx: u16,

    /// Position of rightmost bit to test in key
    key_pos: u8,

    /// Count of bits to test in key (up to 4).
    key_len: u8,

    /// Indexes of subnodes.
    node_idxs: [u16; 16],
}

/// Not-found indicator.
const NONE: u16 = 0xFFFF;

impl<'a, K, T> DecodeTrie<'a, K, T>
where
    K: Into<usize> + Default + BitAndAssign + BitOrAssign,
    T: 'a
{
    #[allow(unused_mut)]
    pub fn build<F>(items: &'a [T], key: F, mask: F) -> Self
    where
        F: Fn(&T) -> K
    {
        let mut nodes = vec![
            Node {
                item_idx: NONE,
                key_pos: 0,
                key_len: 0,
                node_idxs: [NONE; 16]
            }
        ];

        let mut len = size_of::<K>() * 8;
        let mut pos = len - 1;

        let mut todo = !K::default(); // mask of bits needing to be checked

        // loop x
            let mut mask = todo;
            let mut key0 =  K::default();
            let mut key1 = !K::default();

            for (ref item, index) in items.iter().zip(0u16..) {
                let m = mask(item);
                mask &= m;
                let k = key(item);
                key0 |= k;
                key1 &= k;
            }

            // mask now has bits that are j

            let sel = 

            // Find biggest contiguous group of 1s that matter

        // }

        Self {
            items: items,
            nodes: vec![],
            k: PhantomData,
        }
    }
}
*/


