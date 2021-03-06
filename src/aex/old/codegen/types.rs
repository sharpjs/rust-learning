// Type Resolution and Checks
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

use aex::types::{Type, IntSpec, FloatSpec};
use aex::scope::Scope;

// -----------------------------------------------------------------------------
// ResolvedType

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct ResolvedType<'a> {
    pub ty:   &'a Type<'a>,      // Type as written in source
    pub form: TypeForm           // Type reduced to info for typeck and codegen
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum TypeForm {
    Inty    (Option<IntSpec  >), // Int, Ptr
    Floaty  (Option<FloatSpec>), // Float
    Opaque,                      // Array, Union, Struct, Func
}

pub static INT: ResolvedType<'static> = ResolvedType {
    ty:   &Type::Ref("int"),
    form: TypeForm::Inty(None),
};

pub static FLOAT: ResolvedType<'static> = ResolvedType {
    ty:   &Type::Ref("float"),
    form: TypeForm::Floaty(None),
};

pub static OPAQUE: ResolvedType<'static> = ResolvedType {
    ty:   &Type::Ref("unknown"),
    form: TypeForm::Opaque,
};

impl<'a> ResolvedType<'a> {
    pub fn resolve(ty: &'a Type<'a>, scope: &Scope<'a>) -> Result<Self, ()> {
        let res = try!(Self::resolve_form(ty, scope));
        Ok(ResolvedType { ty: ty, form: res })
    }

    fn resolve_form(ty: &Type<'a>, scope: &Scope<'a>) -> Result<TypeForm, ()> {
        match *ty {
            Type::Ref(n) => {
                // For type references, form is that of referenced type
                match scope.types.lookup(n) {
                    Some(ty) => Self::resolve_form(ty, scope),
                    None     => Err(()),
                }
            },
            Type::Int(s) => {
                // For integers, form is inty
                Ok(TypeForm::Inty(s))
            },
            Type::Float(s) => {
                // For floats, form is floaty
                Ok(TypeForm::Floaty(s))
            },
            Type::Ptr(ref ty, _) => {
                // For pointers, form is that of address type (probably inty)
                Self::resolve_form(&**ty, scope)
            },
            _ => {
                // Everything else is opaque
                Ok(TypeForm::Opaque)
            }
        }
    }

    pub fn check_compat(x: Self, y: Self) -> Option<Self> {
        // A type is compatible with itself
        //
        if x.ty as *const _ == y.ty as *const _ {
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

//pub fn check_forms_inty(_a: TypeForm,
//                        _b: TypeForm,
//                        out: TypeForm,
//                        default_width: u8)
//                       -> Option<u8> {
//    match out {
//        TypeForm::Inty(None)    => Some(default_width),
//        TypeForm::Inty(Some(s)) => Some(s.store_width),
//        _                       => None
//    }
//}
//
//pub fn check_forms_inty_extend(src: TypeForm,
//                               dst: TypeForm,
//                               default_width: u8)
//                               -> Option<u8> {
//
//    let sw = check_forms_inty(src, dst, src, default_width);
//    let dw = check_forms_inty(src, dst, dst, default_width);
//
//    // We encode a pair of widths into a single number by addition.
//    // This is not lossy, because widths are powers of 2.
//    // For example: extending u8 to u16 yields a 'width' of 24
//
//    match (sw, dw) {
//        (Some(sw), Some(dw)) if sw < dw => Some(sw + dw),
//        _                               => None,
//    }
//}
//
//// -----------------------------------------------------------------------------
//// Contains - discovers whether a type contains a value
//
//pub trait Contains<T> {
//    fn contains(&self, item: &T) -> Option<bool>;
//    //
//    // Some(true)  => item definitely     in self
//    // Some(false) => item definitely not in self
//    // None        => unknown
//}
//
//impl<T, S> Contains<T> for Option<S> where S: Contains<T> {
//    #[inline]
//    fn contains(&self, item: &T) -> Option<bool> {
//        match *self {
//            Some(ref s) => s.contains(item),
//            None        => None,
//        }
//    }
//}
//
//impl Contains<BigInt> for IntSpec {
//    #[inline]
//    fn contains(&self, value: &BigInt) -> Option<bool> {
//        Some(
//            *value >= self.min_value() &&
//            *value <= self.max_value()
//        )
//    }
//}
//
//impl Contains<BigInt> for TypeForm {
//    #[inline]
//    fn contains(&self, expr: &BigInt) -> Option<bool> {
//        match *self {
//            TypeForm::Inty   (s) => s.contains(expr),
//            TypeForm::Floaty (s) => None,           // Don't know for now
//            TypeForm::Opaque     => Some(false)     // Inexpressable?
//        }
//    }
//}
//
//impl<'a> Contains<BigInt> for ResolvedType<'a> {
//    #[inline(always)]
//    fn contains(&self, item: &BigInt) -> Option<bool> {
//        self.form.contains(item)
//    }
//}
//
