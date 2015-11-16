// Dynamic Trait Equality Deriving
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

use std::any::Any;

// AsAny - upcast from &T to &Any

pub trait AsAny {
    fn as_any(&self) -> &Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &Any { self }
}

// Derive a dynamic Eq for a trait, implemented via Any

#[macro_export]
macro_rules! derive_dynamic_eq {
    ($ty:ident : $eq:ident) => {
        use std::any::Any;

        trait $eq : AsAny {
            fn dynamic_eq(&self, other: &$ty) -> bool;
        }

        impl<T: Any + PartialEq + $ty> $eq for T {
            fn dynamic_eq(&self, other: &$ty) -> bool {
                match other.as_any().downcast_ref::<T>() {
                    Some(x) => self == x,
                    None    => false
                }
            }
        }

        impl<'a> PartialEq for $ty + 'a {
            fn eq(&self, other: &($ty + 'a)) -> bool {
                self.dynamic_eq(other)
            }
        }

        impl<'a> Eq for $ty + 'a {}
    }
}

