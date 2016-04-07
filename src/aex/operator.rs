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
//   - Token::Op      (op)
//   - Expr::BinaryOp (op, lhs, rhs, sel)
//   - Expr::UnaryOp  (op, expr,     sel)

use std::any::Any;
use std::collections::HashMap;
use std::fmt;

use self::Assoc::*;
use self::Fix::*;

#[derive(Clone, Debug)]
pub struct OpTable {
    nonprefix: HashMap<&'static str, Op>, // infix and postfix ops
    prefix:    HashMap<&'static str, Op>, // prefix ops
}

#[derive(Clone, Copy)]
pub struct Op {
    pub chars: &'static str,
    pub prec:  u8,
    pub assoc: Assoc,
    pub fix:   Fix,
    pub eval:  &'static (Any + Sync),
}

impl<T> PartialEq<T> for Op {
    fn eq(&self, other: &T) -> bool {
        false
    }
}

impl Eq for Op { }

impl fmt::Debug for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "")
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Assoc { Left, Right }

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Fix { Prefix, Infix, Postfix }

impl OpTable {
    pub fn new() -> Self {
        OpTable { 
            nonprefix: HashMap::new(),
            prefix:    HashMap::new(),
        }
    }

    pub fn add(&mut self, op: Op) {
        let map = match op.fix {
            Infix | Postfix => &mut self.nonprefix,
            Prefix          => &mut self.prefix,
        };
        map.insert(op.chars, op);
    }

    pub fn get_nonprefix(&self, chars: &str) -> Option<&Op> {
        self.nonprefix.get(chars)
    }

    pub fn get_prefix(&self, chars: &str) -> Option<&Op> {
        self.prefix.get(chars)
    }
}

// For now.  Eventually, targets should provide their available operators.
pub fn create_op_table() -> OpTable {
    let mut table = OpTable::new();
    for &op in OPS { table.add(op) }
    table
}

static OPS: &'static [Op] = &[
    // Postfix Unary
    Op { chars: "++", prec: 10, assoc: Left,  fix: Postfix, eval: &42 },
    Op { chars: "--", prec: 10, assoc: Left,  fix: Postfix, eval: &42 },

    // Prefix Unary
    Op { chars: "!",  prec:  9, assoc: Right, fix: Prefix,  eval: &42 },
    Op { chars: "~",  prec:  9, assoc: Right, fix: Prefix,  eval: &42 },
    Op { chars: "-",  prec:  9, assoc: Right, fix: Prefix,  eval: &42 },
    Op { chars: "+",  prec:  9, assoc: Right, fix: Prefix,  eval: &42 },
    Op { chars: "&",  prec:  9, assoc: Right, fix: Prefix,  eval: &42 },

    // Multiplicative
    Op { chars: "*",  prec:  8, assoc: Left,  fix: Infix,   eval: &42 },
    Op { chars: "/",  prec:  8, assoc: Left,  fix: Infix,   eval: &42 },
    Op { chars: "%",  prec:  8, assoc: Left,  fix: Infix,   eval: &42 },
                                                          
    // Additive                                           
    Op { chars: "+",  prec:  7, assoc: Left,  fix: Infix,   eval: &42 },
    Op { chars: "-",  prec:  7, assoc: Left,  fix: Infix,   eval: &42 },
                                                          
    // Bitwise Shift                                      
    Op { chars: "<<", prec:  6, assoc: Left,  fix: Infix,   eval: &42 },
    Op { chars: ">>", prec:  6, assoc: Left,  fix: Infix,   eval: &42 },
                                                          
    // Bitwise Boolean                                    
    Op { chars: "&",  prec:  5, assoc: Left,  fix: Infix,   eval: &42 },
    Op { chars: "^",  prec:  4, assoc: Left,  fix: Infix,   eval: &42 },
    Op { chars: "|",  prec:  3, assoc: Left,  fix: Infix,   eval: &42 },
                                                          
    // Bitwise Manipulation                               
    Op { chars: ".~", prec:  2, assoc: Left,  fix: Infix,   eval: &42 },
    Op { chars: ".!", prec:  2, assoc: Left,  fix: Infix,   eval: &42 },
    Op { chars: ".+", prec:  2, assoc: Left,  fix: Infix,   eval: &42 },
    Op { chars: ".?", prec:  2, assoc: Left,  fix: Infix,   eval: &42 },

    // Comparison
    Op { chars: "?",  prec:  1, assoc: Left,  fix: Postfix, eval: &42 },
    Op { chars: "<>", prec:  1, assoc: Left,  fix: Infix,   eval: &42 },
    Op { chars: "==", prec:  1, assoc: Left,  fix: Infix,   eval: &42 },
    Op { chars: "!=", prec:  1, assoc: Left,  fix: Infix,   eval: &42 },
    Op { chars: "<" , prec:  1, assoc: Left,  fix: Infix,   eval: &42 },
    Op { chars: "<=", prec:  1, assoc: Left,  fix: Infix,   eval: &42 },
    Op { chars: ">" , prec:  1, assoc: Left,  fix: Infix,   eval: &42 },
    Op { chars: ">=", prec:  1, assoc: Left,  fix: Infix,   eval: &42 },
                                                           
    // Assignment                                          
    Op { chars: "=",  prec:  0, assoc: Right, fix: Infix,   eval: &42 },
];

