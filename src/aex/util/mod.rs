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

pub trait Lookup<K: ?Sized, V: ?Sized> {
    fn lookup(&self, key: &K) -> Option<&V>;
}

#[inline(always)]
pub fn ref_eq<T: ?Sized>(x: &T, y: &T) -> bool {
    x as *const _ == y as *const _
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ref_eq_true() {
        let x = &"".to_string();
        let y = x;
        assert_eq!(ref_eq(x, y), true);
    }

    #[test]
    fn ref_eq_false() {
        let x = &"".to_string();
        let y = &"".to_string();
        assert_eq!(ref_eq(x, y), false);
    }
}

