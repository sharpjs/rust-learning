// Expression Evaluation Helpers
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

use std::fmt::{self, Display, Formatter};

use aex::ast::*;
use aex::pos::Pos;
use aex::scope::Scope;
use aex::types::*;

use super::Context;

// -----------------------------------------------------------------------------
// Evaluator

pub trait Eval {
    fn eval(&self, expr: &Expr, ctx: &mut Context);
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

// Just enough info about a type to enable coercion and opcode selection
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum TypeForm {
    Inty    (Option<(u8, bool)>), // Int, Ptr
    Floaty  (Option<(u8      )>), // Float
    Opaque,                       // Array, Union, Struct, Func
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
            // Form is that of referenced type
            match scope.types.lookup(n) {
                Some(ty) => resolve_type_form(ty, scope),
                None     => Err(n),
            }
        },
        Type::Int(None) => {
            // Form is arbitrary integer
            Ok(TypeForm::Inty(None))
        },
        Type::Int(Some(s)) => {
            // Form is specified integer
            let IntSpec { store_width, signed, .. } = s;
            let s =     ( store_width, signed     );
            Ok(TypeForm::Inty(Some(s)))
        },
        Type::Ptr(ref ty, _) => {
            // Form is that of address type
            resolve_type_form(&**ty, scope)
        },
        _ => {
            // Everything else is opaque
            Ok(TypeForm::Opaque)
        }
    }
}

