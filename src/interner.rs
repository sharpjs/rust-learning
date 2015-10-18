// String Interner
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

use std::collections::HashMap;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct StrId(usize);

impl From<usize> for StrId {
    #[inline]
    fn from(n: usize) -> Self { StrId(n) }
}

impl Into<usize> for StrId {
    #[inline]
    fn into(self) -> usize { self.0 }
}

#[derive(Clone)]
pub struct Interner {
    map: HashMap <String, usize>,
    vec: Vec     <String>
}

impl Interner {
    pub fn new() -> Self {
        Interner {
            map: HashMap::new(),
            vec: Vec    ::new()
        }
    }

    pub fn intern<S: AsRef<str>>(&mut self, s: S) -> StrId
    {
        let s = s.as_ref();

        // If an interned copy exists, return it
        if let Some(&id) = self.map.get(s) {
            return id.into();
        }

        // Else, intern a copy
        let id = self.add(s);
        self.map.insert(s.into(), id.0);
        id
    }

    #[inline]
    pub fn add<S: AsRef<str>>(&mut self, s: S) -> StrId {
        let id = self.vec.len();
        self.vec.push(s.as_ref().into());
        id.into()
    }

    #[inline]
    pub fn get(&self, id: StrId) -> &str {
        &self.vec[id.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern() {
        let mut t = Interner::new();

        let a = t.intern("Hello");
        let b = t.intern(&"Hello".to_string());

        assert!   (a == b);
        assert_eq!("Hello", t.get(a));
    }

    #[test]
    fn add() {
        let mut t = Interner::new();

        let a = t.add("Hello");
        let b = t.add(&"Hello".to_string());

        assert!   (a != b);
        assert_eq!("Hello", t.get(a));
        assert_eq!("Hello", t.get(b));
    }
}

