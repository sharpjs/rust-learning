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
use std::ops::Deref;

use self::Bob::*;

/// A pointer to an immutable value that is either borrowed or boxed.
#[derive(Clone, Debug)]
pub enum Bob<'a, T: ?Sized + 'a> {
    /// Borrowed value.
    Borrowed(&'a T),
    /// Owned value.
    Owned(Box<T>),
}

impl<'a, T> Bob<'a, T> {
    /// Returns a `Bob<T>` that borrows this instance's value.
    pub fn dup(&self) -> Bob<T> {
        Borrowed(&*self)
    }
}

impl<'a, T: ?Sized + 'a> AsRef<T> for Bob<'a, T> {
    fn as_ref(&self) -> &T {
        &*self
    }
}

impl<'a, T: ?Sized + 'a> Borrow<T> for Bob<'a, T> {
    fn borrow(&self) -> &T {
        &*self
    }
}

impl<'a, T: ?Sized + 'a> Deref for Bob<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match *self {
            Borrowed(b)  => b,
            Owned(ref o) => &*o,
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO
}

