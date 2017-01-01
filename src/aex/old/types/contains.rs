// Contains - Check if value is in type
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

pub trait Contains<T> {
    fn contains(&self, item: &T) -> Option<bool>;
    //
    // Some(true)  => item definitely     in self
    // Some(false) => item definitely not in self
    // None        => unknown
}

impl<T, S> Contains<T> for Option<S> where S: Contains<T> {
    #[inline]
    fn contains(&self, item: &T) -> Option<bool> {
        match *self {
            Some(ref s) => s.contains(item),
            None        => None,
        }
    }
}

