// Type Resolution
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

use num::{BigInt /*, ToPrimitive*/};

use aex::types::contains::Contains;
use aex::types::int::IntSpec;
use aex::types::float::FloatSpec;
use aex::types::Type;
//use aex::pos::Source;

// Result of a type resolution
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct TypeRes<'a> {
    pub def:  &'a Type<'a>,         // Type definition
    pub form: TypeForm              // Type reduced to basic info for typecheck
}

// Basic equivalence and size information for a type
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum TypeForm {
    Inty    (Option<  IntSpec>),    // Int, Ptr
    Floaty  (Option<FloatSpec>),    // Float
    Opaque,                         // Array, Union, Struct, Func
}

#[cfg(test)]
mod tests {
}

//impl<'a> TypeRes<'a> {
//    pub fn is_scalar(&self) -> bool {
//        is!(*self => Int(_))
//    }
//
//    pub fn value_width(&self) -> Option<u8> {
//        match *self {
//            Int(Some(IntSpec { value_width, .. })) => Some(value_width),
//            _                                      => None
//        }
//    }
//
//    pub fn store_width(&self) -> Option<u8> {
//        match *self {
//            Int(Some(IntSpec { store_width, .. })) => Some(store_width),
//            _                                      => None
//        }
//    }
//}

impl<'a> TypeRes<'a> {
    pub fn check_compat(x: Self, y: Self) -> Option<Self> {
        // A type is compatible with itself
        //
        if x.def as *const _ == y.def as *const _ {
            return Some(x);
        }
    
        // Otherwise, two types are compatible if:
        //   - they are of the same form, and
        //   - at least one is unbounded.
        //
        match (x.form, y.form) {
            (TypeForm::Inty(xf), TypeForm::Inty(yf)) => {
                match (xf, yf) {
                    (_, None) => Some(x),
                    (None, _) => Some(y),
                    _         => None,
                }
            },
            (TypeForm::Floaty(xf), TypeForm::Floaty(yf)) => {
                match (xf, yf) {
                    (_, None) => Some(x),
                    (None, _) => Some(y),
                    _         => None,
                }
            },
            _ => None
        }
    }

    pub fn check_extend(x: Self, y: Self) -> Option<Self> {
        // Type A is extendible to type B if:
        //   - A and B are of the same form, and
        //   - neither A nor B is unbounded, and
        //   - A is narrower or same width as B.
        //
        match (x.form, y.form) {
            (TypeForm::Inty(xf), TypeForm::Inty(yf)) => {
                match (xf, yf) {
                    (Some(xf), Some(yf))
                        if xf.value_width <= yf.value_width
                        && xf.store_width <= yf.store_width
                        && xf.signed      == yf.signed
                      => Some(y),
                    _ => None
                }
            },
            (TypeForm::Floaty(xf), TypeForm::Floaty(yf)) => {
                match (xf, yf) {
                    (Some(xf), Some(yf))
                        if xf.value_width <= yf.value_width
                        && xf.store_width <= yf.store_width
                      => Some(y),
                    _ => None
                }
            },
            _ => None
        }
    }
}

impl<'a> Contains<BigInt> for TypeRes<'a> {
    #[inline(always)]
    fn contains(&self, item: &BigInt) -> Option<bool> {
        self.form.contains(item)
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

