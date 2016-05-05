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

pub mod context;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use aex::pos::{Pos, Source};
use aex::types::Type;
use aex::types::form::TypeForm;

use self::Fixity::*;
use self::context::Context;

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct OperatorTable<V> {
    nonprefix: HashMap<&'static str, Operator<V>>, // infix and postfix ops
    prefix:    HashMap<&'static str, Operator<V>>, // prefix ops
}

impl<V> OperatorTable<V> {
    pub fn new() -> Self {
        OperatorTable { 
            nonprefix: HashMap::new(),
            prefix:    HashMap::new(),
        }
    }

    pub fn add(&mut self, op: Operator<V>) {
        let map = match op.fixity {
            Infix | Postfix => &mut self.nonprefix,
            Prefix          => &mut self.prefix,
        };
        map.insert(op.chars, op);
    }

    pub fn get_nonprefix(&self, chars: &str) -> Option<&Operator<V>> {
        self.nonprefix.get(chars)
    }

    pub fn get_prefix(&self, chars: &str) -> Option<&Operator<V>> {
        self.prefix.get(chars)
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct Operator<V> {
    pub chars:  &'static str,
    pub prec:   u8,
    pub assoc:  Assoc,
    pub fixity: Fixity,
    pub disp:   Dispatch<V>
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Assoc { Left, Right }

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Fixity { Prefix, Infix, Postfix }

impl<V> Operator<V> {
    pub fn new(chars:  &'static str,
               prec:   u8,
               assoc:  Assoc,
               fixity: Fixity,
               disp:   Dispatch<V>)
              -> Self {
        Operator {
            chars:  chars,
            prec:   prec,
            assoc:  assoc,
            fixity: fixity,
            disp:   disp
        }
    }
}

// -----------------------------------------------------------------------------

pub enum Dispatch<V> {
    Unary  ( UnaryDispatch<V>),
    Binary (BinaryDispatch<V>),
}

impl<V> fmt::Debug for Dispatch<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str(match *self {
            Dispatch::Unary  (..) => "Unary",
            Dispatch::Binary (..) => "Binary",
        })
    }
}

// -----------------------------------------------------------------------------

pub trait Const {
    type Expr;
    fn    new_const (Self::Expr) -> Self;
    fn     is_const (&self     ) -> bool;
    fn unwrap_const ( self     ) -> Self::Expr; // or panic
}

// -----------------------------------------------------------------------------

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Operand<'a, V> {
    pub value:  V,
    pub ty:     TypePtr<'a>,
    pub source: Source<'a>,
}

pub type TypePtr<'a> = Cow<'a, Type<'a, 'a>>;

// -----------------------------------------------------------------------------

macro_rules! def_dispatch {
    { $name:ident ( $($arg:ident),+ ) : $imp:ident } => {
        pub type $imp<V> = for<'a>
            fn ($($arg: Operand<'a, V>),+, ctx: &mut Context<'a>)
            -> Result<Operand<'a, V>, ()>;

        pub struct $name<V> {
            const_op:     Option<$imp<V>>,
            implicit_op:  Option<$imp<V>>,
            explicit_ops: HashMap<&'static str, $imp<V>>,
        }

        impl<V: Const> $name<V> {
            pub fn new() -> Self {
                $name {
                    const_op:     None,
                    implicit_op:  None,
                    explicit_ops: HashMap::new()
                }
            }

            pub fn dispatch<'a>(&self,
                                sel: Option<&str>,
                                $($arg: Operand<'a, V>),+,
                                ctx: &mut Context<'a>)
                               -> Result<Operand<'a, V>, ()> {
                // Get implementation
                let op =
                    if let Some(s) = sel {
                        self.explicit_ops.get(s)
                    } else if true $(&& $arg.value.is_const())+ {
                        self.const_op.as_ref()
                    } else {
                        self.implicit_op.as_ref()
                    };

                // Invoke implementation
                match op {
                    Some(op) => op($($arg),+, ctx),
                    None     => panic!(),
                }
            }
        }
    }
}

def_dispatch! {  UnaryDispatch (a   ) :  UnaryOp }
def_dispatch! { BinaryDispatch (a, b) : BinaryOp }

// -----------------------------------------------------------------------------

macro_rules! op {
    ($name:ident ( $($arg:ident),+ ) :
     $opcodes:expr, $default:expr, $ret:ident :
     $mode_ck:expr, $type_ck:expr, $form_ck:expr) => {
        pub fn $name<'a, V>($($arg: Operand<'a, V>),+, ctx: Context<'a>)
                           -> Result<Operand<'a, V>, ()> {
            // Value/mode check
            // - Does the target op support these values / addressing modes?
            if !$mode_ck($(&$arg.value),+) {
                ctx.out.log.err_no_op_for_addr_modes(Pos::bof("a"));
                return Err(())
            }

            // Get forms before we lose ownership of types
            let forms = ($($arg.ty.form()),+,);

            // Type check
            // - Does the target op take operands of these types?
            // - What type should the result be?
            let ty = match $type_ck($($arg.ty),+) {
                Some(ty) => ty,
                None => {
                    ctx.out.log.err_incompatible_types(Pos::bof("a"));
                    return Err(())
                }
            };

            // Pre-assemble result
            let ret = Operand { value: $ret.value, ty: ty, source: $ret.source };

            // Unpack forms saved earlier
            let ($($arg),+,) = forms;

            // Form check
            // - Does the target op take operands of these storage widths?
            // - What is the width of the opcode that should be used?
            let width = match $form_ck($($arg),+, ret.ty.form(), $default) {
                Some(w) => w,
                None => {
                    ctx.out.log.err_no_op_for_operand_types(Pos::bof("a"));
                    return Err(())
                }
            };

            // Opcode lookup
            let op = match select_opcode(width, $opcodes) {
                Some(op) => op,
                None     => {
                    ctx.out.log.err_no_op_for_operand_sizes(Pos::bof("a"));
                    return Err(())
                }
            };

            //// Emit
            //ctx.out.asm.$write(op, $(&$n),+);

            // Cast result to checked type
            Ok(ret)
        }
    }
}

pub type OpcodeTable = &'static [(u8, &'static str)];

fn select_opcode(ty_width: u8, ops: OpcodeTable) -> Option<&'static str> {
    for &(op_width, op) in ops {
        if op_width == ty_width { return Some(op) }
    }
    None
}

// -----------------------------------------------------------------------------

op! { add  (d, s) : ADD,  32, d : check_values_2, check_types_2, check_forms_2 }
op! { adda (d, s) : ADDA, 32, d : check_values_2, check_types_2, check_forms_2 }

static ADD: OpcodeTable = &[
    (32, "add.l"),
];

static ADDA: OpcodeTable = &[
    (16, "adda.w"),
    (32, "adda.l"),
];

fn check_values_2<V>(a: &V, b: &V) -> bool {
    panic!()
}

fn check_types_2<'a>(a: TypePtr<'a>, b: TypePtr<'a>) -> Option<TypePtr<'a>> {
    panic!()
}

fn check_forms_2(a: TypeForm, b: TypeForm,
                 ret: TypeForm,
                 default: u8)
                -> Option<u8> {
    Some(default)
}

// Move this stuff somewhere eventually.
//
//pub fn create_op_table<V>() -> OperatorTable<V> {
//    let mut table = OperatorTable::new();
//    for &op in OPS { table.add(op) }
//    table
//}
//
//static OPS: &'static [Operator<V>] = &[
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

