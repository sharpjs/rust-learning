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

mod cf; // Freescale ColdFire

pub use std::fmt::{Debug, Display};
pub use std::hash::Hash;

pub use self::cf::ColdFire;

//use aex::operator::*;
//use aex::types::Type;

pub trait Target {
    type String:    Clone + Copy + Eq + PartialEq + Hash + Debug + AsRef<str>;
    type Source:    Clone + Copy + Eq + PartialEq        + Debug;
    type Operator:  Clone        + Eq + PartialEq        + Debug;

    // Target-specific information in an operand
    //type Term: Constness<Expr=Self::Expr>;
    //type Expr;
    //type Operand;

    //fn init_operators(&self, &mut OperatorTable<Self::Term>);
}


