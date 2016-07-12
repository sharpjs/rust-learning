// Target Architectures
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

use std::fmt::Debug;

use aex::ast::Expr;
use aex::context::Context;
use aex::operator::OperatorTable;
use aex::scope::Scope;
use aex::symbol::Symbol;
use aex::types::res::ResolvedType;
use aex::util::Lookup;
use aex::value::Value;

// Target modules
mod cf;     // Freescale ColdFire
mod test;   // For testing; does not generate output

// Target value types
pub use self::cf   ::CfValue;
pub use self::test ::TestValue;

// Target objects
pub const COLDFIRE:    &'static Target = &cf   ::ColdFire;
pub const TEST_TARGET: &'static Target = &test ::TestTarget;

pub trait Target : Debug {

    fn operators(&self) -> &OperatorTable { panic!() }

    fn root_scope(&self) -> &Scope<'static> { panic!() }

    fn eval<'a>(&self, expr: &Expr<'a>, ctx: Context<'a>) -> Value<'a> { panic!() }
}

// Maybe instead a trait Scoped that gives access to symbols() and types() ?

impl<'a> Lookup<str, Symbol<'a>> for Target + 'a {
    #[inline(always)]
    fn lookup(&self, name: &str) -> Option<&Symbol<'a>> {
        self.root_scope().symbols.lookup(name)
    }
}

impl<'a> Lookup<str, ResolvedType<'a>> for Target + 'a {
    #[inline(always)]
    fn lookup(&self, name: &str) -> Option<&ResolvedType<'a>> {
        self.root_scope().types.lookup(name)
    }
}

