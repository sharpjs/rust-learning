// Types
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

pub mod contains;
//pub mod builtin;
pub mod float;
pub mod form;
pub mod int;
//pub mod res;

use std::rc::Rc;
use num::{/*BigInt,*/ BigUint};

use aex::mem::Name;
use aex::pos::Source;
//use aex::types::contains::Contains;
use aex::types::float::FloatSpec;
use aex::types::form::TypeForm;
use aex::types::int::IntSpec;
//use aex::util::ref_eq;

// Type expression
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Type {
    Ident   (Source, Name, Option<Rc<Type>>),
    Int     (Source, Option<IntSpec>),
    Float   (Source, Option<FloatSpec>),
    Array   (Source, Box<Type>, Option<BigUint>),
    Ptr     (Source, Box<Type>, Box<Type>),
    Struct  (Source, Vec<Member>),
    Union   (Source, Vec<Member>),
    Func    (Source, Vec<Member>, Vec<Member>),
}

// Complex type member
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Member (Name, Type);

impl Type {
    pub fn form(&self) -> TypeForm {
        match *self {
            Type::Int   (_, s)              => TypeForm::Inty   (s),
            Type::Float (_, s)              => TypeForm::Floaty (s),
            Type::Ptr   (_, ref t, _)       => t.form(),
            Type::Ident (_, _, Some(ref t)) => t.form(),
            Type::Ident (_, n, None)        => panic_unresolved(),
            _                               => TypeForm::Opaque,
        }
    }

    pub fn as_resolved(&self) -> &Self {
        let mut ty = self;
        while let Type::Ident(_, _, Some(ref t)) = *ty {
            ty = t
        }
        ty
    }

//    pub fn check_compat(x: &Self, y: &Self) -> CheckResult {
//        // If both x and y are type identifiers, use nominal compatibility.
//        // x and y are compatible only if they resolve to the same type.
//        //
//        match (x, y) {
//            (&Type::Ident(_, _, Some(x)),
//             &Type::Ident(_, _, Some(y))) => {
//                if ref_eq(x, y) {
//                    // Identify same type; compatible
//                    return CheckResult::Left
//                } else {
//                    // Not compatible
//                    return CheckResult::None
//                }
//            },
//            _ => ()
//        }
//
//        // Otherwise, use structural compatibility.  x and y are compatible
//        // only if their resolved types have identical structure.
//        //
//        let x = x.as_resolved();
//        let y = y.as_resolved();
//
//        match (x, y) {
//            // Unbounded scalars
//
//            (&Type::Int(_, None), &Type::Int(..     )) => CheckResult::Right,
//            (&Type::Int(_, None), &Type::Ptr(..     )) => CheckResult::Right,
//            (&Type::Int(..     ), &Type::Int(_, None)) => CheckResult::Left,
//            (&Type::Ptr(..     ), &Type::Int(_, None)) => CheckResult::Left,
//
//            (&Type::Float(_, None), &Type::Float(..     )) => CheckResult::Right,
//            (&Type::Float(..     ), &Type::Float(_, None)) => CheckResult::Left,
//
//            // Bounded scalars
//
//            (&Type::Int(_, Some(x)),
//             &Type::Int(_, Some(y))) => {
//                CheckResult::left_if(x == y)
//            },
//
//            (&Type::Float(_, Some(x)),
//             &Type::Float(_, Some(y))) => {
//                CheckResult::left_if(x == y)
//            },
//
//            // Composite
//
//            (&Type::Array(_, ref x_ty, ref x_len),
//             &Type::Array(_, ref y_ty, ref y_len)) => {
//                if *x_len == *y_len {
//                    Self::check_compat(&**x_ty, &**y_ty)
//                } else {
//                    CheckResult::None
//                }
//            },
//
//            (&Type::Ptr(_, ref x_addr_ty, ref x_data_ty),
//             &Type::Ptr(_, ref y_addr_ty, ref y_data_ty)) => {
//                let addr_ck = Self::check_compat(&**x_addr_ty, &**y_addr_ty);
//                let data_ck = Self::check_compat(&**x_data_ty, &**y_data_ty);
//                CheckResult::same_or_none(addr_ck, data_ck)
//            },
//
//            (&Type::Struct(_, ref x_members),
//             &Type::Struct(_, ref y_members)) => {
//                Self::check_compat_members(x_members, y_members)
//            },
//
//            (&Type::Union(_, ref x_members),
//             &Type::Union(_, ref y_members)) => {
//                Self::check_compat_members(x_members, y_members)
//            },
//
//            (&Type::Func(_, ref x_args, ref x_rets),
//             &Type::Func(_, ref y_args, ref y_rets)) => {
//                let args_ck = Self::check_compat_members(x_args, y_args);
//                let rets_ck = Self::check_compat_members(x_rets, y_rets);
//                CheckResult::same_or_none(args_ck, rets_ck)
//            },
//
//            _ => CheckResult::None
//        }
//    }
//
//    fn check_compat_members(x: &[Member], y: &[Member]) -> CheckResult {
//        // Member lists must have same length.
//        if x.len() != y.len() {
//            return CheckResult::None
//        }
//
//        let mut ret = None;
//
//        for pair in x.iter().zip(y.iter()) {
//            let (&Member(x_name, ref x_ty),
//                 &Member(y_name, ref y_ty)) = pair;
//
//            // Must have same member names in the same order.
//            if x_name != y_name {
//                return CheckResult::None
//            }
//
//            // Must have compatible member types
//            let ck = Self::check_compat(x_ty, y_ty);
//            if ck == CheckResult::None {
//                return CheckResult::None
//            }
//
//            // Left/rightness of member checks must agree
//            match ret {
//                None               => ret = Some(ck),
//                Some(r) if r == ck => {/*continue*/},
//                _                  => return CheckResult::None,
//            }
//        }
//
//        ret.unwrap_or(CheckResult::Left)
//    }
//
////    pub fn check_extend(x: Self, y: Self) -> Option<Self> {
////        // Type A is extendible to type B if:
////        //   - A and B are of the same form, and
////        //   - neither A nor B is unbounded, and
////        //   - A is narrower or same width as B.
////        //
////        match (x.form, y.form) {
////            (TypeForm::Inty(xf), TypeForm::Inty(yf)) => {
////                match (xf, yf) {
////                    (Some(xf), Some(yf))
////                        if xf.value_width <= yf.value_width
////                        && xf.store_width <= yf.store_width
////                        && xf.signed      == yf.signed
////                      => Some(y),
////                    _ => None
////                }
////            },
////            (TypeForm::Floaty(xf), TypeForm::Floaty(yf)) => {
////                match (xf, yf) {
////                    (Some(xf), Some(yf))
////                        if xf.value_width <= yf.value_width
////                        && xf.store_width <= yf.store_width
////                      => Some(y),
////                    _ => None
////                }
////            },
////            _ => None
////        }
////    }
}

//impl<'a: 's> Contains<BigInt> for Type {
//    #[inline(always)]
//    fn contains(&self, item: &BigInt) -> Option<bool> {
//        self.form().contains(item)
//    }
//}

fn panic_unresolved() -> ! {
    panic!("Attempted to use unresolved type identifier.")
}

// -----------------------------------------------------------------------------

//#[derive(Clone, Copy, Eq, PartialEq, Debug)]
//pub enum CheckResult { None, Left, Right }
//
//impl CheckResult {
//    #[inline(always)]
//    pub fn left_if(cond: bool) -> Self {
//        if cond { CheckResult::Left } else { CheckResult::Right }
//    }
//
//    #[inline(always)]
//    pub fn same_or_none(x: Self, y: Self) -> Self {
//        if x == y { x } else { CheckResult::None }
//    }
//}
//
//// -----------------------------------------------------------------------------
//
//#[cfg(test)]
//mod tests {
//}
//
