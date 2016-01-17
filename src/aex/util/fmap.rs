// Mapping over Functors (fmap)
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

pub trait Fmap<T, U, O> {
    fn fmap<F: Fn(T) -> U>(self, F) -> O;
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct Ctx<T, C> (pub T, pub C);

// Arity 0
impl<T, U> Fmap<T, U, ()> for ()  {
    fn fmap<F: Fn(T) -> U>(self, f: F) -> () {
        ()
    }
}

// Arity 1
impl<T, U> Fmap<T, U, (U,)> for (T,)  {
    fn fmap<F: Fn(T) -> U>(self, f: F) -> (U,) {
        (f(self.0),)
    }
}

// Arity 2
impl<T, U> Fmap<T, U, (U, U)> for (T, T)  {
    fn fmap<F: Fn(T) -> U>(self, f: F) -> (U, U) {
        (f(self.0), f(self.1))
    }
}

// Arity 3
impl<T, U> Fmap<T, U, (U, U, U)> for (T, T, T)  {
    fn fmap<F: Fn(T) -> U>(self, f: F) -> (U, U, U) {
        (f(self.0), f(self.1), f(self.2))
    }
}

// Arity 4
impl<T, U> Fmap<T, U, (U, U, U, U)> for (T, T, T, T)  {
    fn fmap<F: Fn(T) -> U>(self, f: F) -> (U, U, U, U) {
        (f(self.0), f(self.1), f(self.2), f(self.3))
    }
}

// Don't know of any instruction sets with more than 4 operands.

// Any with context
impl<T, U, TM, UM, C> Fmap<T, U, Ctx<UM, C>> for Ctx<TM, C>
where TM: Fmap<T, U, UM> {
    fn fmap<F: Fn(T) -> U>(self, f: F) -> Ctx<UM, C> {
        let um  = self.0.fmap(f);
        let ctx = self.1;
        Ctx(um, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmap_0() {
        assert_eq!( (), ().fmap(negate) );
    }

    #[test]
    fn fmap_1() {
        assert_eq!( (-1,), (1,).fmap(negate) );
    }

    #[test]
    fn fmap_2() {
        assert_eq!( (-1, -2), (1, 2).fmap(negate) );
    }

    #[test]
    fn fmap_3() {
        assert_eq!( (-1, -2, -3), (1, 2, 3).fmap(negate) );
    }

    #[test]
    fn fmap_4() {
        assert_eq!( (-1, -2, -3, -4), (1, 2, 3, 4).fmap(negate) );
    }

    #[test]
    fn fmap_ctx() {
        assert_eq!( Ctx((-1, -2), "a"), Ctx((1, 2), "a").fmap(negate) );
    }

    fn negate(x: u8) -> i16 { -(x as i16) }
}

