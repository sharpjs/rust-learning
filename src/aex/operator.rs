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

use std::collections::HashMap;
use std::fmt;
//use num::BigInt;

//use aex::ast::Expr;
//use aex::pos::*;

use self::Arity::*;
use self::Fixity::*;

#[derive(Debug)]
pub struct OperatorTable<T> {
    nonprefix: HashMap<&'static str, Operator<T>>, // infix and postfix ops
    prefix:    HashMap<&'static str, Operator<T>>, // prefix ops
}

#[derive(Debug)]
pub struct Operator<T> {
    pub chars:  &'static str,
    pub prec:   u8,
    pub assoc:  Assoc,
    pub fixity: Fixity,
    pub arity:  Arity<T>
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Assoc { Left, Right }

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Fixity { Prefix, Infix, Postfix }

use std::marker::PhantomData;
pub enum Arity<T> {
    Unary  (PhantomData<T>), //(Box<for<'a> Fn(Source<'a>, &str, [Operand<'a, T>; 1]) -> Operand<'a, T>>),
    Binary (PhantomData<T>) //(Box<for<'a> Fn(Source<'a>, &str, [Operand<'a, T>; 2]) -> Operand<'a, T>>),
//    BinarY (Box<for<'a> BinaryDispatch<'a, T>>)
}

impl<T> OperatorTable<T> {
    pub fn new() -> Self {
        OperatorTable { 
            nonprefix: HashMap::new(),
            prefix:    HashMap::new(),
        }
    }

    pub fn add(&mut self, op: Operator<T>) {
        let map = match op.fixity {
            Infix | Postfix => &mut self.nonprefix,
            Prefix          => &mut self.prefix,
        };
        map.insert(op.chars, op);
    }

    pub fn get_nonprefix(&self, chars: &str) -> Option<&Operator<T>> {
        self.nonprefix.get(chars)
    }

    pub fn get_prefix(&self, chars: &str) -> Option<&Operator<T>> {
        self.prefix.get(chars)
    }
}

impl<T/*: Constness*/> Operator<T> {
    pub fn new(chars:  &'static str,
               prec:   u8,
               assoc:  Assoc,
               fixity: Fixity,
               arity:  Arity<T>
              ) -> Self {
        Operator {
            chars:  chars,
            prec:   prec,
            assoc:  assoc,
            fixity: fixity,
            arity:  arity
        }
    }

//    pub fn invoke_unary<'a>(&self, src: Source<'a>, sel: &str, args: [Operand<'a, T>; 1]) -> Operand<'a, T> {
//        if args[0].term.is_const() {
//            // do constant op
//        } else {
//            // do asm op
//        }
//        match self.arity {
//            Unary(ref f) => f(src, sel, args),
//            _ => panic!()
//        }
//    }
}

impl<T> fmt::Debug for Arity<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str(match *self {
            Unary  (_) => "Unary",
            Binary (_) => "Binary",
        })
    }
}

//// -----------------------------------------------------------------------------

use std::borrow::Cow;
use aex::types::Type;
use aex::pos::Source;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Operand<'a, T: Const> {
    pub value:  T,
    pub ttype:  Cow<'a, Type<'a, 'a>>,
    pub source: Source<'a>,
}

pub trait Const {
    type Expr;
    fn    new_const (Self::Expr) -> Self;
    fn     is_const (&self     ) -> bool;
    fn unwrap_const ( self     ) -> Self::Expr; // or panic
}

type BinaryImpl<T> = for<'a> fn([Operand<'a, T>; 2]) -> Result<Operand<'a, T>, ()>;

pub struct BinaryDispatch<T: Const> {
    const_op:     Option<BinaryImpl<T>>,
    implicit_op:  Option<BinaryImpl<T>>,
    explicit_ops: HashMap<&'static str, BinaryImpl<T>>,
}

impl<T: Const> BinaryDispatch<T> {
    pub fn invoke<'a>(&self,
                      selector: Option<&str>,
                      operands: [Operand<'a, T>; 2]
                    ) -> Result<Operand<'a, T>, ()> {

        // Get implementation
        let op =
            if let Some(s) = selector {
                self.explicit_ops.get(s).map(|&op| op)
            } else if all_const(&operands) {
                self.const_op
            } else {
                self.implicit_op
            };

        // Invoke implementation
        match op {
            Some(op) => op(operands),
            None     => panic!(),
        }
    }
}

#[inline]
fn all_const<'a, T: Const>(operands: &[Operand<'a, T>]) -> bool {
    operands.iter().all(|o| o.value.is_const())
}

//pub trait BinaryDispatch<'a, T: 'a + Constness> {
//    fn invoke(&self,
//              src:  Source<'a>,
//              sel:  &str,
//              args: [Operand<'a, T>; 2],
//             ) -> Operand<'a, T> {
//        let op =
//            if sel != "" {
//                // Explicitly selected asm operation
//                self.asm_op_for_sel(sel)
//            } else if args[0].term.is_const() && args[1].term.is_const() {
//                // Constant operation
//                let op = self.const_op();
//                panic!()
//            } else {
//                // Auto-selected asm operation
//                let sel = self.sel_for_args(args).unwrap();
//                self.asm_op_for_sel(sel)
//            };
//        panic!()
//    }
//
//    fn const_op       (&self) -> ConstOperation<T>;
//    fn sel_for_args   (&self, [Operand<'a, T>; 2]) -> Option<&'static str>;
//    fn asm_op_for_sel (&self, &str) -> Option<&AsmOperation<T>>;
//
//    fn set_const_op (&mut self, op: ConstOperation<T>);
//    fn set_asm_auto (&mut self, fn([Operand<'a, T>; 2]) -> Option<&'static str>);
//    fn add_asm_op   (&mut self, sel: &'static str, op: AsmOperation<T>);
//}
//
//use std::marker::PhantomData;
//
//pub struct BinaryDispatcher<T> {
//    const_op: ConstOperation<T>
//}
//
//impl<'a, T: 'a + Constness> BinaryDispatch<'a, T> for BinaryDispatcher<T> {
//    fn set_const_op(&mut self, op: ConstOperation<T>) {
//        panic!()
//    }
//
//    fn set_asm_auto(&mut self, f: fn([Operand<'a, T>; 2]) -> Option<&'static str>) {
//        panic!()
//    }
//
//    fn add_asm_op(&mut self, sel: &'static str, op: AsmOperation<T>) {
//        panic!()
//    }
//
//    fn const_op(&self) -> ConstOperation<T> {
//        panic!()
//    }
//
//    fn sel_for_args(&self, args: [Operand<'a, T>; 2]) -> Option<&'static str> {
//        panic!()
//    }
//
//    fn asm_op_for_sel(&self, sel: &str) -> Option<&AsmOperation<T>> {
//        panic!()
//    }
//}
//
//pub struct ConstOperation<T> {
//    //pub check_types:         fn($($n: TypeA<'a>),+) -> Option<TypeA<'a>>,
//    pub eval_int:            fn([BigInt;      2]) -> BigInt,
//    pub eval_float:          fn([f64;         2]) -> f64,
//    pub eval_expr:   for<'a> fn([Expr<'a, T>; 2]) -> Expr<'a, T>,
//}
//
//pub struct AsmOperation<T> {
//    x: PhantomData<T>
//}
//
//pub enum Const {
//    Int   (BigInt),
//    Float (f64),
//    Expr  (Box<fmt::Display>),
//}

//// For now.  Eventually, targets should provide their available operators.
//pub fn create_op_table<T>() -> OperatorTable<T> {
//    let mut table = OperatorTable::new();
//    for &op in OPS { table.add(op) }
//    table
//}

//static OPS: &'static [Operator<T>] = &[
//    // Postfix Unary
//    Operator { chars: "++", prec: 10, assoc: Left,  fixity: Postfix, eval: &42 },
//    Operator { chars: "--", prec: 10, assoc: Left,  fixity: Postfix, eval: &42 },
//
//    // Prefix Unary
//    Operator { chars: "!",  prec:  9, assoc: Right, fixity: Prefix,  eval: &42 },
//    Operator { chars: "~",  prec:  9, assoc: Right, fixity: Prefix,  eval: &42 },
//    Operator { chars: "-",  prec:  9, assoc: Right, fixity: Prefix,  eval: &42 },
//    Operator { chars: "+",  prec:  9, assoc: Right, fixity: Prefix,  eval: &42 },
//    Operator { chars: "&",  prec:  9, assoc: Right, fixity: Prefix,  eval: &42 },
//
//    // Multiplicative
//    Operator { chars: "*",  prec:  8, assoc: Left,  fixity: Infix,   eval: &42 },
//    Operator { chars: "/",  prec:  8, assoc: Left,  fixity: Infix,   eval: &42 },
//    Operator { chars: "%",  prec:  8, assoc: Left,  fixity: Infix,   eval: &42 },
//                                                          
//    // Additive                                           
//    Operator { chars: "+",  prec:  7, assoc: Left,  fixity: Infix,   eval: &42 },
//    Operator { chars: "-",  prec:  7, assoc: Left,  fixity: Infix,   eval: &42 },
//                                                          
//    // Bitwise Shift                                      
//    Operator { chars: "<<", prec:  6, assoc: Left,  fixity: Infix,   eval: &42 },
//    Operator { chars: ">>", prec:  6, assoc: Left,  fixity: Infix,   eval: &42 },
//                                                          
//    // Bitwise Boolean                                    
//    Operator { chars: "&",  prec:  5, assoc: Left,  fixity: Infix,   eval: &42 },
//    Operator { chars: "^",  prec:  4, assoc: Left,  fixity: Infix,   eval: &42 },
//    Operator { chars: "|",  prec:  3, assoc: Left,  fixity: Infix,   eval: &42 },
//                                                          
//    // Bitwise Manipulation                               
//    Operator { chars: ".~", prec:  2, assoc: Left,  fixity: Infix,   eval: &42 },
//    Operator { chars: ".!", prec:  2, assoc: Left,  fixity: Infix,   eval: &42 },
//    Operator { chars: ".+", prec:  2, assoc: Left,  fixity: Infix,   eval: &42 },
//    Operator { chars: ".?", prec:  2, assoc: Left,  fixity: Infix,   eval: &42 },
//
//    // Comparison
//    Operator { chars: "?",  prec:  1, assoc: Left,  fixity: Postfix, eval: &42 },
//    Operator { chars: "<>", prec:  1, assoc: Left,  fixity: Infix,   eval: &42 },
//    Operator { chars: "==", prec:  1, assoc: Left,  fixity: Infix,   eval: &42 },
//    Operator { chars: "!=", prec:  1, assoc: Left,  fixity: Infix,   eval: &42 },
//    Operator { chars: "<" , prec:  1, assoc: Left,  fixity: Infix,   eval: &42 },
//    Operator { chars: "<=", prec:  1, assoc: Left,  fixity: Infix,   eval: &42 },
//    Operator { chars: ">" , prec:  1, assoc: Left,  fixity: Infix,   eval: &42 },
//    Operator { chars: ">=", prec:  1, assoc: Left,  fixity: Infix,   eval: &42 },
//                                                           
//    // Assignment                                          
//    Operator { chars: "=",  prec:  0, assoc: Right, fixity: Infix,   eval: &42 },
//];


            //Expr::Negate     (ref e, None)        => write!(f, "-{}", e),
            //Expr::Complement (ref e, None)        => write!(f, "~{}", e),
            //Expr::Multiply   (ref l, ref r, None) => write!(f, "({} * {})",  l, r),
            //Expr::Divide     (ref l, ref r, None) => write!(f, "({} / {})",  l, r),
            //Expr::Modulo     (ref l, ref r, None) => write!(f, "({} % {})",  l, r),
            //Expr::Add        (ref l, ref r, None) => write!(f, "({} + {})",  l, r),
            //Expr::Subtract   (ref l, ref r, None) => write!(f, "({} - {})",  l, r),
            //Expr::ShiftL     (ref l, ref r, None) => write!(f, "({} << {})", l, r),
            //Expr::ShiftR     (ref l, ref r, None) => write!(f, "({} >> {})", l, r),
            //Expr::BitAnd     (ref l, ref r, None) => write!(f, "({} & {})",  l, r),
            //Expr::BitXor     (ref l, ref r, None) => write!(f, "({} ^ {})",  l, r),
            //Expr::BitOr      (ref l, ref r, None) => write!(f, "({} | {})",  l, r),

