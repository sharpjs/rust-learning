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

use self::Dispatch::*;
use self::Fixity::*;

#[derive(Debug)]
pub struct OperatorTable<T: Const> {
    nonprefix: HashMap<&'static str, Operator<T>>, // infix and postfix ops
    prefix:    HashMap<&'static str, Operator<T>>, // prefix ops
}

#[derive(Debug)]
pub struct Operator<T: Const> {
    pub chars:  &'static str,
    pub prec:   u8,
    pub assoc:  Assoc,
    pub fixity: Fixity,
    pub disp:   Dispatch<T>
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Assoc { Left, Right }

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Fixity { Prefix, Infix, Postfix }

pub enum Dispatch<T: Const> {
    Unary  (()), // TODO
    Binary (BinaryDispatch<T>),
}

impl<T: Const> OperatorTable<T> {
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

impl<T: Const> Operator<T> {
    pub fn new(chars:  &'static str,
               prec:   u8,
               assoc:  Assoc,
               fixity: Fixity,
               disp:   Dispatch<T>
              ) -> Self {
        Operator {
            chars:  chars,
            prec:   prec,
            assoc:  assoc,
            fixity: fixity,
            disp:   disp
        }
    }

//    pub fn invoke_unary<'a>(&self, src: Source<'a>, sel: &str, args: [Operand<'a, T>; 1]) -> Operand<'a, T> {
//        if args[0].term.is_const() {
//            // do constant op
//        } else {
//            // do asm op
//        }
//        match self.disp {
//            Unary(ref f) => f(src, sel, args),
//            _ => panic!()
//        }
//    }
}

impl<T: Const> fmt::Debug for Dispatch<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str(match *self {
            Unary  (_) => "Unary",
            Binary (_) => "Binary",
        })
    }
}

//// -----------------------------------------------------------------------------

use std::borrow::Cow;
use std::marker::PhantomData;
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

#[derive(Clone, Debug)]
pub struct Context<'a> {
    x: PhantomData<&'a ()>
}

type BinaryImpl<T> = for<'a>
    fn ([Operand<'a, T>; 2], Context<'a>)
       -> Result<Operand<'a, T>, ()>;

pub struct BinaryDispatch<T: Const> {
    const_op:     Option<BinaryImpl<T>>,
    implicit_op:  Option<BinaryImpl<T>>,
    explicit_ops: HashMap<&'static str, BinaryImpl<T>>,
}

impl<T: Const> BinaryDispatch<T> {
    pub fn new() -> Self {
        BinaryDispatch {
            const_op:     None,
            implicit_op:  None,
            explicit_ops: HashMap::new()
        }
    }

    pub fn invoke<'a>(&self,
                      selector: Option<&str>,
                      operands: [Operand<'a, T>; 2],
                      context:  Context<'a>,
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
            Some(op) => op(operands, context),
            None     => panic!(),
        }
    }
}

#[inline]
fn all_const<'a, T: Const>(operands: &[Operand<'a, T>]) -> bool {
    operands.iter().all(|o| o.value.is_const())
}

//// -----------------------------------------------------------------------------

use aex::types::form::TypeForm;

pub struct TargetBinaryOp<T: Const> {
    pub default_width: u8,

    pub check_modes:
        for<'a> fn([&T; 2]) -> bool,

    pub check_types:
        for<'a> fn([&Type<'a, 'a>; 2]) -> Option<Cow<'a, Type<'a, 'a>>>,

    pub check_forms:
        fn([TypeForm; 2], u8) -> Option<u8>,
}

impl<T: Const + Clone> TargetBinaryOp<T> {
    pub fn invoke<'a>(&self,
                      operands: [Operand<'a, T>; 2],
                      context:  Context<'a>,
                     ) -> Result<Operand<'a, T>, ()> {
        // Mode check
        let vs = [&operands[0].value, &operands[1].value];
        let ok = (self.check_modes)(vs);
        if !ok {
            return Err(())
        }

        // Type check
        let ty = (self.check_types)([&operands[0].ttype, &operands[1].ttype]);
        let ty = match ty {
            Some(ty) => ty,
            None     => {
                return Err(())
            }
        };

        // Form check
        let width = self.default_width;
        let width = (self.check_forms)([operands[0].ttype.form(),
                                        operands[1].ttype.form()],
                                       width);
        let width = match width {
            Some(w) => w,
            None    => {
                return Err(())
            }
        };

        // Opcode select
        // ?

        // Emit
        // ?

        Ok(Operand {
            value:  operands[0].value.clone(),
            ttype:  ty,
            source: operands[0].source
        })
    }

}

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

