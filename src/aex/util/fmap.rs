// Functors (fmap) for Rust
//
// Copyright (C) 2016 Jeffrey Sharp
//
// This file is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published
// by the Free Software Foundation, either version 3 of the License,
// or (at your option) any later version.
//
// This file is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See
// the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this file.  If not, see <http://www.gnu.org/licenses/>.

pub trait Fmap<T, U> {
    type Out;

    fn fmap<'a, F>(&'a self, F) -> Self::Out
        where F: Fn(&'a T) -> U,
              T: 'a;
}

// LESSON LEARNED: Rust does not (yet) have higher-kinded types (HKT),
// which are necessary to express Functor like it is in Haskell.
// For now, we have to provide both the T and U types.
//
// Rust also does not (yet) have variadic generic tuples.  So, we have
// to implement Fmap for every size of tuple that we need.

// Arity 0
impl<T, U> Fmap<T, U> for [T; 0] {
    type Out = [U; 0];

    #[inline(always)]
    fn fmap<'a, F>(&'a self, f: F) -> Self::Out
        where F: Fn(&'a T) -> U,
              T: 'a
    { [] }
}

// Arity 1
impl<T, U> Fmap<T, U> for [T; 1] {
    type Out = [U; 1];

    #[inline(always)]
    fn fmap<'a, F>(&'a self, f: F) -> Self::Out
        where F: Fn(&'a T) -> U,
              T: 'a
    { [ f(&self[0]) ] }
}

// Arity 2
impl<T, U> Fmap<T, U> for [T; 2]  {
    type Out = [U; 2];

    #[inline]
    fn fmap<'a, F>(&'a self, f: F) -> Self::Out
        where F: Fn(&'a T) -> U,
              T: 'a
    { [ f(&self[0]), f(&self[1]) ] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmap_0() {
        assert_eq!( [0; 0], [0; 0].fmap(negate) );
    }

    #[test]
    fn fmap_1() {
        assert_eq!( [-1], [1].fmap(negate) );
    }

    #[test]
    fn fmap_2() {
        assert_eq!( [-1, -2], [1, 2].fmap(negate) );
    }

    fn negate(x: &u8) -> i16 { -(*x as i16) }
}

