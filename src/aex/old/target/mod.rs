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
use std::ops::Deref;

use aex::ast::Expr;
use aex::context::Context;
use aex::operator::OperatorTable;
use aex::scope::{Scope, Scoped};
use aex::symbol::Symbol;
use aex::types::res::ResolvedType;
use aex::util::Lookup;
use aex::value::Value;

// Target modules
mod cf;     // Freescale ColdFire
mod test;   // For testing; does not generate output

// Target value types
pub use self::cf::   { ColdFire,   CfValue   };
pub use self::test:: { TestTarget, TestValue };

// -----------------------------------------------------------------------------

pub trait Target : Debug {
    fn operators(&self) -> &OperatorTable { panic!() }

    fn root_scope(&self) -> &Scope<'static> { panic!() }

    fn eval<'a>(&self, e: &Expr<'a>, c: Context<'a>) -> Value<'a> { panic!() }
}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct TargetRef<'a> (&'a Target);

impl<'a> TargetRef<'a> {
    #[inline(always)]
    pub fn new(target: &'a Target) -> Self {
        TargetRef(target)
    }

    #[inline(always)]
    pub fn set(&mut self, target: &'a Target) {
        self.0 = target;
    }
}

impl<'a> Deref for TargetRef<'a> {
    type Target = Target + 'a;

    #[inline(always)]
    fn deref(&self) -> &Self::Target { self.0 }
}

impl<'a> Scoped<'a> for TargetRef<'a> {
    #[inline]
    fn symbols(&self) -> &Lookup<str, Symbol<'a>> {
        self
    }

    #[inline]
    fn types(&self) -> &Lookup<str, ResolvedType<'a>> {
        self
    }
}

impl<'a> Lookup<str, Symbol<'a>> for TargetRef<'a> {
    fn lookup(&self, name: &str) -> Option<&Symbol<'a>> {
        self.root_scope().lookup(name)
    }
}

impl<'a> Lookup<str, ResolvedType<'a>> for TargetRef<'a> {
    fn lookup(&self, name: &str) -> Option<&ResolvedType<'a>> {
        self.root_scope().lookup(name)
    }
}

