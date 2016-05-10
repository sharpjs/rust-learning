// Object Identifiers
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

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

// -----------------------------------------------------------------------------

pub type Name = Id<str>;

// -----------------------------------------------------------------------------

pub struct Id<T: ?Sized> (u32, PhantomData<T>);

// #[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
//
// #[derive] doesn't know (yet) to ignore the T from PhantomData<T>.
// Thus we have to implement these traits explicitly.
// This probably will be fixed in a later version of Rust.

impl<T: ?Sized> From<usize> for Id<T> {
    #[inline(always)]
    fn from(n: usize) -> Id<T> {
        Id(n as u32, PhantomData)
    }
}

impl<T: ?Sized> From<Id<T>> for usize {
    #[inline(always)]
    fn from(id: Id<T>) -> usize {
        id.0 as usize
    }
}

impl<T: ?Sized> Clone for Id<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Id(self.0, PhantomData)
    }
}

impl<T: ?Sized> Copy for Id<T> { }

impl<T: ?Sized> PartialEq for Id<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: ?Sized> Eq for Id<T> { }

impl<T: ?Sized> Hash for Id<T> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T: ?Sized> fmt::Display for Id<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}>", self.0)
    }
}

impl<T: ?Sized> fmt::Debug for Id<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}>", self.0)
    }
}

// -----------------------------------------------------------------------------

#[cfg(test)]
pub mod tests {
    use std::marker::PhantomData;
    use super::*;

    pub const NAME_ZERO: Name = Id(0, PhantomData);

    #[test]
    fn id_roundtrip() {
        let n = usize::from(Name::from(42));
        assert_eq!(n, 42);
    }
}

