// Unsafe Pointer
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

use std::borrow::Borrow;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

// -----------------------------------------------------------------------------

/// A raw pointer that is assumed to be safely derefable
/// for its entire lifetime.  Use with care.
///
pub struct Ptr<T: ?Sized> (*const T);

impl<T: ?Sized> Ptr<T> {
    // Like the usual as_ref(), but will promote to any lifetime.
    #[inline]
    pub fn as_ref<'a>(self) -> &'a T {
        unsafe { &*self.0 }
    }
}

impl<'a, T: ?Sized> From<&'a T> for Ptr<T> {
    #[inline]
    fn from(other: &'a T) -> Self {
        Ptr(other)
    }
}

impl<T: ?Sized> Clone for Ptr<T> {
    #[inline]
    fn clone(&self) -> Self {
        Ptr(self.0)
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.0 = source.0
    }
}

impl<T: ?Sized> Copy for Ptr<T> { }

impl<T: ?Sized> Deref for Ptr<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<T: ?Sized> Borrow<T> for Ptr<T> {
    #[inline]
    fn borrow(&self) -> &T {
        self.as_ref()
    }
}

impl<T: ?Sized> PartialEq for Ptr<T> where T: PartialEq {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().eq(other.as_ref())
    }

    #[inline]
    fn ne(&self, other: &Self) -> bool {
        self.as_ref().ne(other.as_ref())
    }
}

impl<T: ?Sized> Eq for Ptr<T> where T: Eq { }

impl<T: ?Sized> Hash for Ptr<T> where T: Hash {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}

impl<T: ?Sized> Display for Ptr<T> where T: Display {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<T: ?Sized> Debug for Ptr<T> where T: Debug {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

