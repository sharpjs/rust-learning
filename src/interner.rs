// AEx - Just a toy language for learning Rust
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
pub struct Symbol(usize);

impl From<usize> for Symbol {
    #[inline]
    fn from(n: usize) -> Self { Symbol(n) }
}

impl Into<usize> for Symbol {
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

    pub fn intern<S: AsRef<str>>(&mut self, name: S) -> Symbol
    {
        let key = name.as_ref();

        // If a symbol exists by that name, return it
        if let Some(&idx) = self.map.get(key) {
            return idx.into();
        }

        // Else, build a new symbol and return it
        let idx = self.vec.len();
        self.vec.push   (key.into()     );
        self.map.insert (key.into(), idx);
        idx.into()
    }

    #[inline]
    pub fn get(&self, id: Symbol) -> &str {
        &self.vec[id.0]
    }
}

#[test]
fn test() {
    let mut t = Interner::new();

    let a = t.intern("Hello");
    let b = t.intern(&"Hello".to_string());

    assert_eq!(a, b);
}

