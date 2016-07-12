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
use aex::operator::OperatorTable;
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

    fn eval<'a>(&self, expr: &Expr<'a>, ctx: Context<'a>) -> Value<'a> { panic!() }
}

