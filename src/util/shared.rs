// Smart Pointer to Shared Immutable Data
//
// This file is part of AEx.
// Copyright (C) 2015 Jeffrey Sharp
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

use std::fmt::{self, Display, Pointer, Formatter};
use std::ops::Deref;
use std::rc::Rc;
use self::Shared::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Shared<'a, T: 'a + ?Sized> {
    Borrowed(&'a T),
    Owned(Rc<T>)
}

impl<'a, T: 'a + ?Sized> From<&'a T> for Shared<'a, T> {
    #[inline(always)]
    fn from(t: &'a T) -> Self { Borrowed(t) }
}

impl<'a, T: 'a + ?Sized> From<Rc<T>> for Shared<'a, T> {
    #[inline(always)]
    fn from(t: Rc<T>) -> Self { Owned(t) }
}

impl<'a, T: 'a> From<T> for Shared<'a, T> {
    #[inline(always)]
    fn from(t: T) -> Self { Owned(Rc::new(t)) }
}

impl<'a, T: 'a + ?Sized> Deref for Shared<'a, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        match self {
            &Borrowed (    t) => t,
            &Owned    (ref t) => t.deref(),
        }
    }
}

impl<'a, T: 'a + ?Sized> Clone for Shared<'a, T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        match self {
            &Borrowed (    t) => Borrowed(t),
            &Owned    (ref t) => Owned(t.clone()),
        }
    }
}

impl<'a, T: 'a + Default> Default for Shared<'a, T> {
    #[inline]
    fn default() -> Self {
        Owned(Default::default())
    }
}

impl<'a, T: 'a + ?Sized + Display> Display for Shared<'a, T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl<'a, T: 'a> Pointer for Shared<'a, T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            &Borrowed (ref t) => Pointer::fmt(t, f),
            &Owned    (ref t) => Pointer::fmt(t, f),
        }
    }
}

