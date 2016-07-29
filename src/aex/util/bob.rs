// Bob ("Borrow or Box") Smart Pointer
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

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{self, Debug, Display, Formatter, Pointer};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use self::Bob::*;

/// A pointer to an immutable value that is either borrowed or boxed.
pub enum Bob<'a, T: ?Sized + 'a> {
    /// Borrowed value.
    Borrowed(&'a T),

    /// Owned value.
    Owned(Box<T>),
}

// Construction ----------------------------------------------------------------

impl<'a, T> Bob<'a, T> {
    /// Returns a `Bob<T>` that borrows this instance's value.
    #[inline(always)]
    pub fn dup(&self) -> Bob<T> {
        Borrowed(&**self)
    }
}

impl<'a, T: ?Sized + 'a> Clone for Bob<'a, T> where T: Clone {
    #[inline]
    fn clone(&self) -> Self {
        Owned(Box::new((&**self).clone()))
    }
}

impl<'a, T: 'a> From<T> for Bob<'a, T> {
    #[inline(always)]
    fn from(t: T) -> Self {
        Owned(Box::new(t))
    }
}

impl<'a, T: ?Sized + 'a> From<&'a T> for Bob<'a, T> {
    #[inline(always)]
    fn from(t: &'a T) -> Self {
        Borrowed(t)
    }
}

// Pointer ---------------------------------------------------------------------

impl<'a, T: ?Sized + 'a> AsRef<T> for Bob<'a, T> {
    #[inline(always)]
    fn as_ref(&self) -> &T {
        &**self
    }
}

impl<'a, T: ?Sized + 'a> Borrow<T> for Bob<'a, T> {
    #[inline(always)]
    fn borrow(&self) -> &T {
        &**self
    }
}

impl<'a, T: ?Sized + 'a> Deref for Bob<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        match *self {
            Borrowed(b)  => b,
            Owned(ref o) => &*o,
        }
    }
}

// Comparison ------------------------------------------------------------------

impl<'a, T: ?Sized + 'a> PartialEq for Bob<'a, T> where T: PartialEq {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        &**self == &**other
    }

    #[inline(always)]
    fn ne(&self, other: &Self) -> bool {
        &**self != &**other
    }
}

impl<'a, T: ?Sized + 'a> Eq for Bob<'a, T> where T: Eq {}

impl<'a, T: ?Sized + 'a> PartialOrd for Bob<'a, T> where T: PartialOrd {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (&**self).partial_cmp(&**other)
    }

    #[inline(always)]
    fn lt(&self, other: &Self) -> bool {
        &**self < &**other
    }

    #[inline(always)]
    fn le(&self, other: &Self) -> bool {
        &**self <= &**other
    }

    #[inline(always)]
    fn gt(&self, other: &Self) -> bool {
        &**self > &**other
    }

    #[inline(always)]
    fn ge(&self, other: &Self) -> bool {
        &**self >= &**other
    }
}

impl<'a, T: ?Sized + 'a> Ord for Bob<'a, T> where T: Ord {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        (&**self).cmp(&**other)
    }
}

impl<'a, T: ?Sized + 'a> Hash for Bob<'a, T> where T: Hash {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        (&**self).hash(state);
    }
}

// Formatting ------------------------------------------------------------------

impl<'a, T: ?Sized + 'a> Display for Bob<'a, T> where T: Display {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<'a, T: ?Sized + 'a> Debug for Bob<'a, T> where T: Debug {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, T: ?Sized + 'a> Pointer for Bob<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Borrowed(b)  => Pointer::fmt(&b, f),
            Owned(ref o) => Pointer::fmt( o, f),
        }
    }
}

// Tests -----------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::Bob;
    use super::Bob::*;

    #[test]
    fn from_t() {
        let val: i32      = 42;
        let bob: Bob<i32> = Bob::from(val);

        match bob {
            Borrowed(b)  => panic!(),
            Owned(ref o) => assert_eq!(**o, 42),
        }
    }

    #[test]
    fn from_ref_t() {
        let val: i32      = 42;
        let bob: Bob<i32> = Bob::from(&val);

        match bob {
            Borrowed(b)  => assert_eq!(*b, val),
            Owned(ref o) => panic!()
        }
    }
}

