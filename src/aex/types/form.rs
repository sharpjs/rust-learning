// Basic Type Forms
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

use num::BigInt;

use aex::types::contains::Contains;
use aex::types::float::FloatSpec;
use aex::types::int::IntSpec;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum TypeForm {
    Inty    (Option<  IntSpec>),    // Int, Ptr
    Floaty  (Option<FloatSpec>),    // Float
    Opaque,                         // Array, Union, Struct, Func
}

impl TypeForm {
    pub fn is_scalar(&self) -> bool {
        match *self {
            TypeForm::Inty   (..) => true,
            TypeForm::Floaty (..) => true,
            _                     => false
        }
    }

    pub fn value_width(&self) -> Option<u8> {
        match *self {
            TypeForm::Inty(Some(IntSpec { value_width, .. })) => {
                Some(value_width)
            },
            TypeForm::Floaty(Some(FloatSpec { value_width, .. })) => {
                Some(value_width)
            },
            _ => None
        }
    }

    pub fn store_width(&self) -> Option<u8> {
        match *self {
            TypeForm::Inty(Some(IntSpec { store_width, .. })) => {
                Some(store_width)
            },
            TypeForm::Floaty(Some(FloatSpec { store_width, .. })) => {
                Some(store_width)
            },
            _ => None
        }
    }
}

impl Contains<BigInt> for TypeForm {
    #[inline]
    fn contains(&self, expr: &BigInt) -> Option<bool> {
        match *self {
            TypeForm::Inty   (s) => s.contains(expr),
            TypeForm::Floaty (s) => None, // Don't know for now
            TypeForm::Opaque     => Some(false)
        }
    }
}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
}

