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

pub trait Fmap<T, U> {
    type Out;

    fn fmap<F: Fn(T) -> U>(self, F) -> Self::Out;
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct Ctx<T, C> (pub T, pub C);

// Arity 0
impl<T, U> Fmap<T, U> for () {
    type Out = ();

    fn fmap<F: Fn(T) -> U>(self, f: F) -> Self::Out {
        ()
    }
}

// Arity 1
impl<T, U> Fmap<T, U> for (T,)  {
    type Out = (U,);

    fn fmap<F: Fn(T) -> U>(self, f: F) -> Self::Out {
        (f(self.0),)
    }
}

// Arity 2
impl<T, U> Fmap<T, U> for (T, T)  {
    type Out = (U, U);

    fn fmap<F: Fn(T) -> U>(self, f: F) -> Self::Out {
        (f(self.0), f(self.1))
    }
}

// Arity 3
impl<T, U> Fmap<T, U> for (T, T, T)  {
    type Out = (U, U, U);

    fn fmap<F: Fn(T) -> U>(self, f: F) -> Self::Out {
        (f(self.0), f(self.1), f(self.2))
    }
}

// Arity 4
impl<T, U> Fmap<T, U> for (T, T, T, T)  {
    type Out = (U, U, U, U);

    fn fmap<F: Fn(T) -> U>(self, f: F) -> Self::Out {
        (f(self.0), f(self.1), f(self.2), f(self.3))
    }
}

// Any arity with context
impl<T, U, M, C> Fmap<T, U> for Ctx<M, C> where M: Fmap<T, U> {
    type Out = Ctx<M::Out, C>;

    fn fmap<F: Fn(T) -> U>(self, f: F) -> Self::Out {
        let Ctx(m,         c) = self;
            Ctx(m.fmap(f), c)
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

