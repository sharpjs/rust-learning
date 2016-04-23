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
pub mod builtin;
pub mod float;
pub mod int;
pub mod res;

use std::rc::Rc;
use num::BigUint;

use aex::pos::Source;
use aex::types::float::FloatSpec;
use aex::types::int::IntSpec;
use aex::util::ref_eq;

// Type expression

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Type<'a> {
    Ident   (Source<'a>, &'a str, Option<Rc<Type<'a>>>),
    Int     (Source<'a>, Option<IntSpec>),
    Float   (Source<'a>, Option<FloatSpec>),
    Array   (Source<'a>, Box<Type<'a>>, Option<BigUint>),
    Ptr     (Source<'a>, Box<Type<'a>>, Box<Type<'a>>),
    Struct  (Source<'a>, Vec<Member<'a>>),
    Union   (Source<'a>, Vec<Member<'a>>),
    Func    (Source<'a>, Vec<Member<'a>>, Vec<Member<'a>>),
}

// Complex type member

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Member<'a> (&'a str, Type<'a>);

// Basic equivalence and size information for a type
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum TypeForm {
    Inty    (Option<  IntSpec>),    // Int, Ptr
    Floaty  (Option<FloatSpec>),    // Float
    Opaque,                         // Array, Union, Struct, Func
}

enum CheckResult { None, Left, Right }

impl<'a> Type<'a> {
    pub fn form(&self) -> TypeForm {
        match *self {
            Type::Int   (_, s)              => TypeForm::Inty   (s),
            Type::Float (_, s)              => TypeForm::Floaty (s),
            Type::Ptr   (_, ref t, _)       => t.form(),
            Type::Ident (_, _, Some(ref t)) => t.form(),
            Type::Ident (_, n, None)        => panic_unresolved(n),
            _                               => TypeForm::Opaque,
        }
    }

    fn as_resolved(&self) -> &Self {
        let mut ty = self;
        while let Type::Ident(_, _, Some(ref t)) = *ty {
            ty = &**t
        }
        ty
    }

    fn check_compat(x: &Self, y: &Self) -> CheckResult {
        // If both x and y are type identifiers, use nominal compatibility.
        // x and y are compatible only if they resolve to the same type.
        //
        match (x, y) {
            (&Type::Ident(_, _, Some(ref x)),
             &Type::Ident(_, _, Some(ref y))) => {
                if ref_eq(&**x, &**y) {
                    // Identify same type; compatible
                    return CheckResult::Left
                } else {
                    // Not compatible
                    return CheckResult::None
                }
            },
            _ => ()
        }

        // Otherwise, use structural compatibility.  x and y are compatible
        // only if their resolved types have identical structure.
        //
        let x = x.as_resolved();
        let y = y.as_resolved();
        match (x, y) {
            (&Type::Int(_, x), &Type::Int(_, y)) => {
                match (x, y) {
                    (x, y) if x == y => CheckResult::Left,
                    (_, None)        => CheckResult::Left,
                    (None, _)        => CheckResult::Right,
                    _                => CheckResult::None,
                }
            },
            (&Type::Float(_, x), &Type::Float(_, y)) => {
                match (x, y) {
                    (x, y) if x == y => CheckResult::Left,
                    (_, None)        => CheckResult::Left,
                    (None, _)        => CheckResult::Right,
                    _                => CheckResult::None,
                }
            },
            (&Type::Array(_, ref x_ty, ref x_len),
             &Type::Array(_, ref y_ty, ref y_len)) => {
                if ref_eq(&**x_ty, &**y_ty) && *x_len == *y_len {
                    CheckResult::Left
                } else {
                    CheckResult::None
                }
            },
            (&Type::Ptr(_, ref x_addr_ty, ref x_data_ty),
             &Type::Ptr(_, ref y_addr_ty, ref y_data_ty)) => {
                if ref_eq(&**x_addr_ty, &**y_addr_ty) &&
                   ref_eq(&**x_data_ty, &**y_data_ty) {
                    CheckResult::Left
                } else {
                    CheckResult::None
                }
            },
            (&Type::Struct(_, ref x_members),
             &Type::Struct(_, ref y_members)) => {
                if Self::eq_members(x_members, y_members) {
                    CheckResult::Left
                } else {
                    CheckResult::None
                }
            },
            (&Type::Union(_, ref x_members),
             &Type::Union(_, ref y_members)) => {
                if Self::eq_members(x_members, y_members) {
                    CheckResult::Left
                } else {
                    CheckResult::None
                }
            },
            (&Type::Func(_, ref x_args, ref x_rets),
             &Type::Func(_, ref y_args, ref y_rets)) => {
                if Self::eq_members(x_args, y_args) &&
                   Self::eq_members(x_rets, y_rets) {
                    CheckResult::Left
                } else {
                    CheckResult::None
                }
            },
            _ => CheckResult::None
        }
    }

    fn eq_members(x: &[Member<'a>], y: &[Member<'a>]) -> bool {
        x.len() == y.len() &&
        x.iter().zip(y.iter()).all(|(x, y)| x == y)
    }

//    pub fn check_extend(x: Self, y: Self) -> Option<Self> {
//        // Type A is extendible to type B if:
//        //   - A and B are of the same form, and
//        //   - neither A nor B is unbounded, and
//        //   - A is narrower or same width as B.
//        //
//        match (x.form, y.form) {
//            (TypeForm::Inty(xf), TypeForm::Inty(yf)) => {
//                match (xf, yf) {
//                    (Some(xf), Some(yf))
//                        if xf.value_width <= yf.value_width
//                        && xf.store_width <= yf.store_width
//                        && xf.signed      == yf.signed
//                      => Some(y),
//                    _ => None
//                }
//            },
//            (TypeForm::Floaty(xf), TypeForm::Floaty(yf)) => {
//                match (xf, yf) {
//                    (Some(xf), Some(yf))
//                        if xf.value_width <= yf.value_width
//                        && xf.store_width <= yf.store_width
//                      => Some(y),
//                    _ => None
//                }
//            },
//            _ => None
//        }
//    }
}

fn panic_unresolved(name: &str) -> ! {
    panic!("Attempted to use unresolved type identifier '{}'", name)
}

