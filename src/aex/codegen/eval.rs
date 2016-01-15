// Expression Evaluation Helpers
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

use std::fmt::{self, Display, Formatter};

use aex::ast::*;
use aex::pos::Pos;
use aex::scope::Scope;
use aex::types::*;

use super::Context;

// -----------------------------------------------------------------------------
// Eval - interface to an evaluator

pub trait Eval {
    fn eval<'cg, 'str>
           (self: &    Self,
            expr: &    Expr   <     'str>,
            ctx:  &mut Context<'cg, 'str>);
}

// -----------------------------------------------------------------------------
// Operand - a machine location with its analyzed type and source position

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Operand<'a, L: 'a + Display> {
    pub loc: L          ,   // Machine location
    pub ty:  TypeA <'a> ,   // Analyzed type
    pub pos: Pos   <'a> ,   // Source position
}

impl<'a, L: 'a + Display> Operand<'a, L> {
    pub fn new(loc: L, ty: TypeA<'a>, pos: Pos<'a>) -> Self {
        Operand { loc: loc, ty: ty, pos: pos }
    }
}

impl<'a, L: 'a + Display> Display for Operand<'a, L> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.loc, f)
    }
}

// -----------------------------------------------------------------------------
// TypeA - an analyzed type

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct TypeA<'a> {
    pub ty:   &'a Type<'a>, // Type as written in source
    pub form: TypeForm      // Type reduced to info for typeck and codegen
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum TypeForm {
    Inty    (Option<IntSpec  >), // Int, Ptr
    Floaty  (Option<FloatSpec>), // Float
    Opaque,                      // Array, Union, Struct, Func
}

pub fn analyze_type
    <'a>
    (ty:    &'a Type <'a>,
     scope: &'a Scope<'a>)
    -> Result<TypeA<'a>, &'a str> {

    let res = try!(resolve_type_form(ty, scope));

    Ok(TypeA { ty: ty, form: res })
}

fn resolve_type_form
    <'a>
    (ty:    &'a Type  <'a>,
     scope: &'a Scope <'a>)
    -> Result<TypeForm, &'a str> {

    match *ty {
        Type::Ref(n) => {
            // For type references, form is that of referenced type
            match scope.types.lookup(n) {
                Some(ty) => resolve_type_form(ty, scope),
                None     => Err(n),
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
            resolve_type_form(&**ty, scope)
        },
        _ => {
            // Everything else is opaque
            Ok(TypeForm::Opaque)
        }
    }
}

// -----------------------------------------------------------------------------
// Contains - discovers whether a type contains a value

pub trait Contains<T> {
    fn contains(&self, item: &T) -> Option<bool>;
    //
    // Some(true)  => item definitely     in self
    // Some(false) => item definitely not in self
    // None        => unknown
}

impl<'a, T, S> Contains<T> for Option<S> where S: Contains<T> {
    #[inline]
    fn contains(&self, item: &T) -> Option<bool> {
        match *self {
            Some(ref s) => s.contains(item),
            None        => None,
        }
    }
}

impl<'a> Contains<Expr<'a>> for IntSpec {
    fn contains(&self, expr: &Expr<'a>) -> Option<bool> {
        match *expr {
            Expr::Int(ref v) => {
                let ok = *v >= self.min_value()
                      && *v <= self.max_value();
                Some(ok)
            },
            _ => None,
        }
    }
}

impl<'a> Contains<Expr<'a>> for TypeForm {
    fn contains(&self, expr: &Expr<'a>) -> Option<bool> {
        match *self {
            TypeForm::Inty   (s) => s.contains(expr),
            TypeForm::Floaty (s) => None,           // Don't know for now
            TypeForm::Opaque     => Some(false)     // Inexpressable?
        }
    }
}

impl<'a, 'b> Contains<Expr<'a>> for TypeA<'b> {
    #[inline(always)]
    fn contains(&self, item: &Expr<'a>) -> Option<bool> {
        self.form.contains(item)
    }
}

