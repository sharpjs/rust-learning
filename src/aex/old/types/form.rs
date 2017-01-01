// Basic Type Forms
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

use num::BigInt;

use aex::types::contains::Contains;
use aex::types::float::FloatSpec;
use aex::types::int::IntSpec;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct TypeInfo {
    pub form:  TypeForm,    // element form
    pub step:  usize,       // if element is a ptr, size of inc/dec
    pub count: usize,       // number of elements; 0 = unknown
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum TypeForm {
    Inty    (Option<IntSpec  >),    // Int, Ptr
    Floaty  (Option<FloatSpec>),    // Float
    Opaque  (Option<usize    >),    // Array, Union, Struct, Func
}

impl TypeInfo {
    pub fn size_bytes(&self) -> usize {
        self.form.size_bytes() * self.count
    }
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

    pub fn size_bytes(&self) -> usize {
        match *self {
            TypeForm::Inty(Some(IntSpec { store_width, .. })) => {
                bits_to_bytes(store_width)
            },
            TypeForm::Floaty(Some(FloatSpec { store_width, .. })) => {
                bits_to_bytes(store_width)
            },
            TypeForm::Opaque(Some(size)) => {
                size
            },
            _ => 0,
        }
    }
}

#[inline]
fn bits_to_bytes(bits: u8) -> usize {
    (bits as usize).next_power_of_two() >> 3
}

impl Contains<BigInt> for TypeForm {
    #[inline]
    fn contains(&self, expr: &BigInt) -> Option<bool> {
        match *self {
            TypeForm::Inty   (s) => s.contains(expr),
            TypeForm::Floaty (s) => None, // Don't know for now
            TypeForm::Opaque (_) => Some(false)
        }
    }
}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
}

