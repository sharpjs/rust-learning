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

use std::marker::PhantomData;

// -----------------------------------------------------------------------------

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Id<T: ?Sized> (usize, PhantomData<T>);

pub type Name = Id<str>;

impl<T: ?Sized> Clone for Id<T> {
    fn clone(&self) -> Self {
        Id(self.0, PhantomData)
    }

    fn clone_from(&mut self, source: &Self) {
        self.0 = source.0
    }
}

impl<T: ?Sized> Copy for Id<T> { }

impl<T: ?Sized> From<usize> for Id<T> {
    fn from(n: usize) -> Id<T> { Id(n, PhantomData) }
}

impl<T: ?Sized> From<Id<T>> for usize {
    fn from(id: Id<T>) -> usize { id.0 }
}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::Name;

    #[test]
    fn id_roundtrip() {
        let n = usize::from(Name::from(42));
        assert_eq!(n, 42);
    }
}

