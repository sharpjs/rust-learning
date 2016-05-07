// Aex Memory Utilities Module
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

pub mod arena;
//pub mod interner;

use std::marker::PhantomData;

pub type Name = Id<str>;

// -----------------------------------------------------------------------------
// Id

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Id<T: ?Sized> (u32, PhantomData<T>);

impl<T: ?Sized> From<u32> for Id<T> {
    fn from(n: u32) -> Self { Id(n, PhantomData) }
}

impl<T: ?Sized> From<Id<T>> for u32 {
    fn from(id: Id<T>) -> Self { id.0 }
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use super::Name;

    #[test]
    fn roundtrip() {
        let n = u32::from(Name::from(42));
        assert_eq!(n, 42);
    }
}

