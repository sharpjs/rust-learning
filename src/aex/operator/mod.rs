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

//pub mod context;

//use std::borrow::Cow;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};

//use aex::pos::{Pos, Source};
//use aex::types::Type
//use aex::types::form::TypeForm;
//use aex::util::ref_eq;
use aex::value::Value;

use self::Assoc::*;
use self::Arity::*;
//use self::context::Context;

// Temporary
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Context;

// -----------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OperatorTable {
    map: HashMap<&'static str, OperatorEntry>,
}

impl OperatorTable {
    pub fn new() -> Self {
        OperatorTable { map: HashMap::new() }
    }

    pub fn add(&mut self, op: Operator) {
        let entry = self.map
            .entry(op.chars)
            .or_insert_with(|| OperatorEntry::new());

        match (op.arity, op.assoc) {
            (Unary(..), Right) => entry.prefix = Some(op),
            _                  => entry.suffix = Some(op),
        }
    }

    pub fn map(&self) -> &HashMap<&'static str, OperatorEntry> {
        &self.map
    }

    pub fn get<S>(&self, chars: S) -> Option<&OperatorEntry>
    where S: Borrow<str> {
        self.map.get(chars.borrow())
    }
}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct OperatorEntry {
    pub prefix: Option<Operator>,   // prefix
    pub suffix: Option<Operator>,   // infix or postfix
}

impl OperatorEntry {
    pub fn new() -> Self {
        OperatorEntry { prefix: None, suffix: None }
    }
}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Operator {
    pub chars: &'static str,
    pub prec:  u8,
    pub assoc: Assoc,
    pub arity: Arity,
}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Assoc { Left, Right }

// -----------------------------------------------------------------------------

// HACK: Rust won't derive Clone, PartialEq, or Debug, because of the
// existential lifetime 'a.  I think this might be a compiler bug.
// For now, I'll just implement Clone explicitly.
//
//#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arity {
    Unary  (for<'a> fn(Context, Value<'a>           ) -> Value<'a>),
    Binary (for<'a> fn(Context, Value<'a>, Value<'a>) -> Value<'a>),
}

impl Clone for Arity {
    fn clone(&self) -> Self { *self }

    fn clone_from(&mut self, source: &Self) { *self = *source }
}

impl Copy for Arity { }

impl PartialEq for Arity {
    fn eq(&self, other: &Self) -> bool {
        match (*self, *other) {
            (Unary (l), Unary (r)) => l as *const () == r as *const (),
            (Binary(l), Binary(r)) => l as *const () == r as *const (),
            _ => false,
        }
    }
}

impl Eq for Arity { }

impl Debug for Arity {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            Unary  (m) => write!(f,  "Unary({:p})", m as *const ()),
            Binary (m) => write!(f, "Binary({:p})", m as *const ()),
        }
    }
}

//// -----------------------------------------------------------------------------
//
//pub enum Dispatch<V> {
//    Unary  ( UnaryDispatch<V>),
//    Binary (BinaryDispatch<V>),
//}
//
//impl<V> fmt::Debug for Dispatch<V> {
//    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
//        f.write_str(match *self {
//            Unary  (..) => "Unary",
//            Binary (..) => "Binary",
//        })
//    }
//}
//
//// -----------------------------------------------------------------------------
//
//pub trait Const {
//    type Expr;
//    fn    new_const (Self::Expr) -> Self;
//    fn     is_const (&self     ) -> bool;
//    fn unwrap_const ( self     ) -> Self::Expr; // or panic
//}
//
//// -----------------------------------------------------------------------------
//
//#[derive(Clone, Eq, PartialEq, Debug)]
//pub struct Operand<'a, V> {
//    pub value:  V,
//    pub ty:     TypePtr<'a>,
//    pub source: Source<'a>,
//}
//
//pub type TypePtr<'a> = Cow<'a, Type<'a, 'a>>;
//
//// -----------------------------------------------------------------------------
//
//macro_rules! def_dispatch {
//    { $name:ident ( $($arg:ident),+ ) : $imp:ident, $new:ident, $disp:ident } => {
//        pub type $imp<V> = for<'a>
//            fn ($($arg: Operand<'a, V>),+, ctx: &mut Context<'a>)
//            -> Result<Operand<'a, V>, ()>;
//
//        pub fn $new<V: Const>(chars:  &'static str,
//                              prec:   u8,
//                              assoc:  Assoc,
//                              fixity: Fixity)
//                             -> Operator<V> {
//            Operator::<V>::new(
//                chars, prec, assoc, fixity,
//                Dispatch::$disp($name::<V>::new())
//            )
//        }
//
//        pub struct $name<V> {
//            const_op:     Option<$imp<V>>,
//            implicit_op:  Option<$imp<V>>,
//            explicit_ops: HashMap<&'static str, $imp<V>>,
//        }
//
//        impl<V: Const> $name<V> {
//            pub fn new() -> Self {
//                $name {
//                    const_op:     None,
//                    implicit_op:  None,
//                    explicit_ops: HashMap::new()
//                }
//            }
//
//            pub fn dispatch<'a>(&self,
//                                sel: Option<&str>,
//                                $($arg: Operand<'a, V>),+,
//                                ctx: &mut Context<'a>)
//                               -> Result<Operand<'a, V>, ()> {
//                // Get implementation
//                let op =
//                    if let Some(s) = sel {
//                        self.explicit_ops.get(s)
//                    } else if true $(&& $arg.value.is_const())+ {
//                        self.const_op.as_ref()
//                    } else {
//                        self.implicit_op.as_ref()
//                    };
//
//                // Invoke implementation
//                match op {
//                    Some(op) => op($($arg),+, ctx),
//                    None     => panic!(),
//                }
//            }
//        }
//    }
//}
//
//def_dispatch! {  UnaryDispatch (a   ) :  UnaryOp,  unary_op,  Unary }
//def_dispatch! { BinaryDispatch (a, b) : BinaryOp, binary_op, Binary }
//
//// -----------------------------------------------------------------------------
//
//macro_rules! op {
//    { $name:ident ( $($arg:ident),+ )
//        : $opcodes:expr, $default:expr, $ret:ident
//        : $mode_ck:expr, $type_ck:expr, $form_ck:expr
//    } => {
//        pub fn $name<'a, V>($($arg: Operand<'a, V>),+, ctx: Context<'a>)
//                           -> Result<Operand<'a, V>, ()> {
//            // Value/mode check
//            // - Does the target op support these values / addressing modes?
//            if !$mode_ck($(&$arg.value),+) {
//                ctx.out.log.err_no_op_for_addr_modes(Pos::bof("a"));
//                return Err(())
//            }
//
//            // Get forms before we lose ownership of types
//            let forms = ($($arg.ty.form()),+,);
//
//            // Type check
//            // - Does the target op take operands of these types?
//            // - What type should the result be?
//            let ty = match $type_ck($($arg.ty),+) {
//                Some(ty) => ty,
//                None => {
//                    ctx.out.log.err_incompatible_types(Pos::bof("a"));
//                    return Err(())
//                }
//            };
//
//            // Pre-assemble result
//            let ret = Operand { value: $ret.value, ty: ty, source: $ret.source };
//
//            // Unpack forms saved earlier
//            let ($($arg),+,) = forms;
//
//            // Form check
//            // - Does the target op take operands of these storage widths?
//            // - What is the width of the opcode that should be used?
//            let width = match $form_ck($($arg),+, ret.ty.form(), $default) {
//                Some(w) => w,
//                None => {
//                    ctx.out.log.err_no_op_for_operand_types(Pos::bof("a"));
//                    return Err(())
//                }
//            };
//
//            // Opcode lookup
//            let op = match select_opcode(width, $opcodes) {
//                Some(op) => op,
//                None     => {
//                    ctx.out.log.err_no_op_for_operand_sizes(Pos::bof("a"));
//                    return Err(())
//                }
//            };
//
//            //// Emit
//            //ctx.out.asm.$write(op, $(&$n),+);
//
//            // Cast result to checked type
//            Ok(ret)
//        }
//    }
//}
//
//pub type OpcodeTable = &'static [(u8, &'static str)];
//
//fn select_opcode(ty_width: u8, ops: OpcodeTable) -> Option<&'static str> {
//    for &(op_width, op) in ops {
//        if op_width == ty_width { return Some(op) }
//    }
//    None
//}
//
//// -----------------------------------------------------------------------------
//
//op! { add  (d, s) : ADD,  32, d : check_values_2, check_types_2, check_forms_2 }
//op! { adda (d, s) : ADDA, 32, d : check_values_2, check_types_2, check_forms_2 }
//
//static ADD: OpcodeTable = &[
//    (32, "add.l"),
//];
//
//static ADDA: OpcodeTable = &[
//    (16, "adda.w"),
//    (32, "adda.l"),
//];
//
//fn check_values_2<V>(a: &V, b: &V) -> bool {
//    panic!()
//}
//
//fn check_types_2<'a>(a: TypePtr<'a>, b: TypePtr<'a>) -> Option<TypePtr<'a>> {
//    panic!()
//}
//
//fn check_forms_2(a: TypeForm, b: TypeForm,
//                 ret: TypeForm,
//                 default: u8)
//                -> Option<u8> {
//    Some(default)
//}
//
//// -----------------------------------------------------------------------------
//
//pub fn def_builtin_operators<V: Const>(table: &mut OperatorTable<V>) {
//    for op in builtin_operators::<V>() { table.add(op) }
//}
//
//fn builtin_operators<'a, V: Const>() -> Vec<Operator<V>> {
//    vec![
//        // Postfix Unary
//         unary_op::<V>( "++" , 10 , Left  , Postfix ),
//         unary_op::<V>( "--" , 10 , Left  , Postfix ),
//        // Prefix Unary
//         unary_op::<V>( "!"  ,  9 , Right , Prefix  ),
//         unary_op::<V>( "~"  ,  9 , Right , Prefix  ),
//         unary_op::<V>( "-"  ,  9 , Right , Prefix  ),
//         unary_op::<V>( "+"  ,  9 , Right , Prefix  ),
//         unary_op::<V>( "&"  ,  9 , Right , Prefix  ),
//        // Multiplicative
//        binary_op::<V>( "*"  ,  8 , Left  , Infix   ),
//        binary_op::<V>( "/"  ,  8 , Left  , Infix   ),
//        binary_op::<V>( "%"  ,  8 , Left  , Infix   ),
//        // Additive                          
//        binary_op::<V>( "+"  ,  7 , Left  , Infix   ),
//        binary_op::<V>( "-"  ,  7 , Left  , Infix   ),
//        // Bitwise Shift                      
//        binary_op::<V>( "<<" ,  6 , Left  , Infix   ),
//        binary_op::<V>( ">>" ,  6 , Left  , Infix   ),
//        // Bitwise Boolean                   
//        binary_op::<V>( "&"  ,  5 , Left  , Infix   ),
//        binary_op::<V>( "^"  ,  4 , Left  , Infix   ),
//        binary_op::<V>( "|"  ,  3 , Left  , Infix   ),
//        // Bitwise Manipulation              
//        binary_op::<V>( ".~" ,  2 , Left  , Infix   ),
//        binary_op::<V>( ".!" ,  2 , Left  , Infix   ),
//        binary_op::<V>( ".+" ,  2 , Left  , Infix   ),
//        binary_op::<V>( ".?" ,  2 , Left  , Infix   ),
//        // Comparison                     
//        binary_op::<V>( "?"  ,  1 , Left  , Postfix ),
//        binary_op::<V>( "<>" ,  1 , Left  , Infix   ),
//        binary_op::<V>( "==" ,  1 , Left  , Infix   ),
//        binary_op::<V>( "!=" ,  1 , Left  , Infix   ),
//        binary_op::<V>( "<"  ,  1 , Left  , Infix   ),
//        binary_op::<V>( "<=" ,  1 , Left  , Infix   ),
//        binary_op::<V>( ">"  ,  1 , Left  , Infix   ),
//        binary_op::<V>( ">=" ,  1 , Left  , Infix   ),
//        // Assignment                                              
//        binary_op::<V>( "="  ,  0 , Right , Infix   ),
//    ]
//}

// This needs to go somewhere:
//
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

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    #[test]
    fn ne() {
        let fx = &FA;
        let fy = &FB;

        match (fx, fy) {
            (&F(px), &F(py)) => {
                assert!(px as *const () != py as *const ());
                return;
            }
        }
    }

    #[test]
    fn eq() {
        let fx = &FA;
        let fy = &FA;

        match (fx, fy) {
            (&F(px), &F(py)) => {
                assert!(px as *const () == py as *const ());
                return;
            }
        }
    }

    #[test]
    fn size_a() {
        assert_eq!(8, size_of::<F>());
    }

    struct F(fn());
    static FA: F = F(a);
    static FB: F = F(b);

    #[inline(never)]
    fn a() { println!("a");  println!("a");  println!("a"); }

    #[inline(never)]
    fn b() { println!("b") }
}

