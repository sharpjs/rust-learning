// Freescale ColdFire Target
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

//mod loc;
//mod code_gen;

use std::marker::PhantomData;

use aex::operator::{Op, OpTable};
use aex::operator::Assoc::*;
use aex::operator::Fixity::*;
use aex::operator::Arity::*;
use aex::pos::Pos;
use aex::target::Target;

pub struct ColdFire<'a> {
    _x: PhantomData<&'a ()>
}

impl<'a> ColdFire<'a> {
    pub fn new() -> Self {
        ColdFire { _x: PhantomData }
    }
}

impl<'a> Target for ColdFire<'a> {
    type Term = CfTerm<'a>;

    fn init_operators(&self, operators: &mut OpTable<Self::Term>) {
        operators.add(Op {
            chars: "+", prec: 7, assoc: Left, fixity: Infix, arity: Binary(
                Box::new(|p, s, a| CfTerm::B)
            )
        });
    }
}

// Temporary
pub enum CfTerm<'a> { A(&'a str), B }

fn add<'a>(pos: &Pos, sel: &str, args: [CfTerm<'a>; 2]) -> CfTerm<'a> {
    CfTerm::B
}

