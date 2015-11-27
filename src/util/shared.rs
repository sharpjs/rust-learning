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

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{self, Display, Pointer, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::rc::Rc;
use self::Shared::*;

pub type SharedStr = Shared<'static, str, String>;

#[derive(Debug)]
pub enum Shared<'a,
    B: 'a + ?Sized,
    O: 'a + ?Sized + Borrow<B> = B
> {
    Borrowed(&'a B),
    Owned(Rc<O>)
}

impl<'a,
    B: 'a + ?Sized,
    O: 'a +  Sized + Borrow<B> + Default
>
Default for Shared<'a, B, O> {
    #[inline(always)]
    fn default() -> Self {
        Owned(Default::default())
    }
}

impl<'a,
    B: 'a + ?Sized,
    O: 'a + ?Sized + Borrow<B>
>
From<&'a B> for Shared<'a, B, O> {
    #[inline(always)]
    fn from(x: &'a B) -> Self { Borrowed(x) }
}

impl<'a,
    B: 'a + ?Sized,
    O: 'a + ?Sized + Borrow<B>
>
From<Rc<O>> for Shared<'a, B, O> {
    #[inline(always)]
    fn from(x: Rc<O>) -> Self { Owned(x) }
}

impl<'a,
    B: 'a + ?Sized,
    O: 'a + ?Sized + Borrow<B>
>
Clone for Shared<'a, B, O> {
    #[inline(always)]
    fn clone(&self) -> Self {
        match self {
            &Borrowed (    x) => Borrowed(x),
            &Owned    (ref x) => Owned(x.clone()),
        }
    }
}

impl<'a,
    B: 'a + ?Sized,
    O: 'a + ?Sized + Borrow<B>
>
Deref for Shared<'a, B, O> {
    type Target = B;

    #[inline]
    fn deref(&self) -> &B {
        match self {
            &Borrowed (    x) => x,
            &Owned    (ref x) => x.deref().borrow(),
        }
    }
}

impl<'a,
    B: 'a + ?Sized,
    O: 'a + ?Sized + Borrow<B>
>
Borrow<B> for Shared<'a, B, O> {
    #[inline(always)]
    fn borrow(&self) -> &B { self.deref() }
}

impl<'a,
    B: 'a + ?Sized,
    O: 'a + ?Sized + Borrow<B>
>
AsRef<B> for Shared<'a, B, O> {
    #[inline(always)]
    fn as_ref(&self) -> &B { self.deref() }
}

impl<'a,
    B: 'a + ?Sized + PartialEq,
    O: 'a + ?Sized + Borrow<B>,
    T: 'a + ?Sized + Borrow<B>
>
PartialEq<T> for Shared<'a, B, O> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.deref().eq(other.borrow())
    }
}

impl<'a,
    B: 'a + ?Sized + Eq,
    O: 'a + ?Sized + Borrow<B>
>
Eq for Shared<'a, B, O> {}

impl<'a,
    B: 'a + ?Sized + PartialOrd,
    O: 'a + ?Sized + Borrow<B>,
    T: 'a + ?Sized + Borrow<B>
>
PartialOrd<T> for Shared<'a, B, O> {
    #[inline]
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        self.deref().partial_cmp(other.borrow())
    }
}

impl<'a,
    B: 'a + ?Sized + Ord,
    O: 'a + ?Sized + Borrow<B>
>
Ord for Shared<'a, B, O> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.deref().cmp(other.deref())
    }
}

impl<'a,
    B: 'a + ?Sized + Hash,
    O: 'a + ?Sized + Borrow<B>
>
Hash for Shared<'a, B, O> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl<'a,
    B: 'a + ?Sized + Display,
    O: 'a + ?Sized + Borrow<B>
>
Display for Shared<'a, B, O> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self.deref(), f)
    }
}

impl<'a,
    B: 'a,
    O: 'a + Borrow<B>
>
Pointer for Shared<'a, B, O> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            &Borrowed (ref x) => Pointer::fmt(x, f),
            &Owned    (ref x) => Pointer::fmt(x, f),
        }
    }
}

