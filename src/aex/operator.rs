// Operators
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

// FUTURE:
//   - Use operator table
//   - Targets add their own operators at startup
//   - Token::Operator(str)
//   - Expr::BinaryOp (op, lhs, rhs, sel)
//   - Expr::UnaryOp  (op, expr,     sel)

use std::collections::HashMap;

use self::Assoc::*;
use self::Fix::*;

#[derive(Clone, Debug)]
pub struct OperatorTable (
    HashMap<&'static str, Operator>
);

#[derive(Clone, Copy, Debug)]
pub struct Operator {
    pub chars: &'static str,
    pub prec:  u8,
    pub assoc: Assoc,
    pub fix:   Fix,
  //pub eval:  &'static Any,
}

#[derive(Clone, Copy, Debug)]
pub enum Assoc { Left, Right }

#[derive(Clone, Copy, Debug)]
pub enum Fix { Prefix, Infix, Postfix }

impl OperatorTable {
    pub fn new() -> Self {
        OperatorTable(HashMap::new())
    }

    pub fn add(&mut self, op: Operator) {
        self.0.insert(op.chars, op);
    }

    pub fn get(&self, chars: &str) -> Option<&Operator> {
        self.0.get(chars)
    }
}

fn create_op_table() -> OperatorTable {
    let mut table = OperatorTable::new();
    table.add(EQUAL);
    table
}

static EQUAL: Operator = Operator {
    chars: "=",
    prec:  0,
    assoc: Right,
    fix:   Infix,
};

