// Utilities
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

use std::fmt::{self, Formatter};

// -----------------------------------------------------------------------------

pub trait DisplayWith<C: ?Sized> {
    fn fmt(&self, f: &mut Formatter, c: &C) -> fmt::Result;
}

pub type WriteFn<V> = fn(f: &mut Formatter, v: &V) -> fmt::Result;

// -----------------------------------------------------------------------------

pub trait Lookup<K: ?Sized, V: ?Sized> {
    fn lookup(&self, key: &K) -> Option<&V>;
}

// -----------------------------------------------------------------------------

#[inline(always)]
pub fn ref_eq<T: ?Sized>(x: &T, y: &T) -> bool {
    x as *const _ == y as *const _
}

// -----------------------------------------------------------------------------

#[macro_export]
macro_rules! result {
    ($cond:expr) => (
        if $cond { Ok(()) } else { Err(()) }
    );
    ($cond:expr, $ok:expr) => (
        if $cond { Ok($ok) } else { Err(()) }
    );
    ($cond:expr, $ok:expr, $err:expr) => (
        if $cond { Ok($ok) } else { Err($err) }
    );
}

// -----------------------------------------------------------------------------

// From http://stackoverflow.com/a/28392068/142138
#[macro_export]
macro_rules! hash_map {
    ($( $key:expr => $val:expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    static X: str = *"xax";
    static Y: str = *"yay";

    #[test]
    fn ref_eq_true() {
        let x = &X;
        let y =  x;
        assert_eq!(ref_eq(x, y), true);
    }

    #[test]
    fn ref_eq_false_by_ptr() {
        let x = &X[1..2];
        let y = &Y[1..2];
        assert_eq!(ref_eq(x, y), false);
    }

    #[test]
    fn ref_eq_false_by_len() {
        let x = &X[1..2];
        let y = &X[1.. ];
        assert_eq!(ref_eq(x, y), false);
    }
}

