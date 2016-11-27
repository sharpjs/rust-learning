// Abstract Syntax Tree
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

use std::fmt::{self, Formatter};
use num::BigInt;

use aex::asm::AsmFlavor;
use aex::source::Source;
use aex::types::Type;
use aex::util::DisplayWith;

// -----------------------------------------------------------------------------
// Statements

pub type Ast<'a> = Vec<Stmt<'a>>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Stmt<'a> {
    // Composite
    Block   (Ast     <'a>), // { ... }

    // Declaration
    TypeDef (TypeDef <'a>), // type foo = u8
    Label   (Label   <'a>), // foo:
    DataLoc (DataLoc <'a>), // foo: u8
    DataVal (DataVal <'a>), // foo: u8 = 42
    Func    (Func    <'a>), // foo: u8 -> u8 { ... }

    // Execution
    Eval    (Expr    <'a>), // x + 42
    Loop    (Loop    <'a>), // loop         { ... }
    If      (If      <'a>), // if     x > 0 { ... } else { ... }
    While   (While   <'a>), // while  x > 0 { ... }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TypeDef<'a> {
    pub id:  Id<'a>,
    pub ty:  Type<'a>,
    pub src: Source<'a>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Label<'a> {
    pub id:  Id<'a>,
    pub src: Source<'a>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DataLoc<'a> {
    pub id:  Id<'a>,
    pub ty:  Type<'a>,
    pub src: Source<'a>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DataVal<'a> {
    pub id:  Id<'a>,
    pub ty:  Type<'a>,
    pub val: Expr<'a>,
    pub src: Source<'a>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Func<'a> {
    pub id:  Id<'a>,
    pub ty:  Type<'a>,
    pub ast: Ast<'a>,
    pub src: Source<'a>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Loop<'a> {
    pub ast: Ast<'a>,
    pub src: Source<'a>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct If<'a> {
    pub cond:     Cond<'a>,
    pub if_true:  Ast<'a>,
    pub if_false: Ast<'a>,
    pub src:      Source<'a>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct While<'a> {
    pub cond: Cond<'a>,
    pub ast:  Ast<'a>,
    pub src:  Source<'a>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Cond<'a> {
    pub sel:  Id<'a>,
    pub expr: Option<Expr<'a>>,
    pub src:  Source<'a>,
}

// -----------------------------------------------------------------------------
// Expressions

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expr<'a> {
    // CORE OPERATORS
    // 12: Atomic
    Id      (Id         <'a>),  // identifier
    Str     (StrLit     <'a>),  // string literal
    Int     (IntLit     <'a>),  // number literal
    // 11: Postfix
    PostInc (UnaryExpr  <'a>),  // v++      post-increment
    PostDec (UnaryExpr  <'a>),  // v--      post-decrement
    Call    ,
    Elem    (BinaryExpr <'a>),  // x[y]
    Member  (BinaryExpr <'a>),  // x.y
    // 10: Prefix
    PreInc  (UnaryExpr  <'a>),
    PreDec  (UnaryExpr  <'a>),
    Ref     (UnaryExpr  <'a>),  // &v
    Deref   (UnaryExpr  <'a>),  // *v
    Clr     (UnaryExpr  <'a>),  // !v
    Not     (UnaryExpr  <'a>),  // ~v
    Neg     (UnaryExpr  <'a>),  // -v
    // 9: Cast/Convert
    Cast    (CastExpr   <'a>),  // v :  t
    Conv    (CastExpr   <'a>),  // v :> t
    // 8: Multiplicative
    Mul     (BinaryExpr <'a>),  // v * v
    Div     (BinaryExpr <'a>),  // v / v
    Mod     (BinaryExpr <'a>),  // v % v
    // 7: Additive
    Add     (BinaryExpr <'a>),  // v + v
    Sub     (BinaryExpr <'a>),  // v - v
    // 6: Shift/Rotate
    Shl     (BinaryExpr <'a>),  // v << v   shift left
    Shr     (BinaryExpr <'a>),  // v >> v   shift right
    Rol     (BinaryExpr <'a>),  // v <<| v  rotate left
    Ror     (BinaryExpr <'a>),  // v |>> v  rotate right
    Rcl     (BinaryExpr <'a>),  // v <<+ v  rotate left through carry
    Rcr     (BinaryExpr <'a>),  // v +>> v  rotate right through carry
    // 5: And
    And     (BinaryExpr <'a>),  // v & v    bitwise and
    // 4: Xor
    Xor     (BinaryExpr <'a>),  // v ^ v    bitwise xor
    // 3: And
    Or      (BinaryExpr <'a>),  // v | v    bitwise or
    // 2: Compare
    Cmp     (BinaryExpr <'a>),  // v <> v   compare
    Test    (UnaryExpr  <'a>),  // v?       test
    // 1: Conditional
    Eq      (BinaryExpr <'a>),  // v == v   equal
    Ne      (BinaryExpr <'a>),  // v != v   not equal
    Lt      (BinaryExpr <'a>),  // v <  v   less than
    Le      (BinaryExpr <'a>),  // v <= v   less than or equal
    Gt      (BinaryExpr <'a>),  // v >  v   greater than
    Ge      (BinaryExpr <'a>),  // v >= v   greater than or equal
    // 0: Assignment
    Assign  (BinaryExpr <'a>),  // v = v    assign

    // Other
    Unary   (UnaryExpr  <'a>),  // 1-ary operation
    Binary  (BinaryExpr <'a>),  // 2-ary operation
}

impl<'a> Expr<'a> {
    pub fn src(&self) -> Source<'a> {
        Source::BuiltIn // TODO
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UnaryExpr<'a> {
    pub expr: Box<Expr<'a>>,    // value
    pub sel:  Id<'a>,           // operation selector
    pub src:  Source<'a>,       // source location
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BinaryExpr<'a> {
    pub lhs:  Box<Expr<'a>>,    // left value
    pub rhs:  Box<Expr<'a>>,    // right value
    pub sel:  Id<'a>,           // operation selector
    pub src:  Source<'a>,       // source location
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CastExpr<'a> {
    pub expr: Box<Expr<'a>>,    // value
    pub ty:   Type<'a>,         // type
    pub src:  Source<'a>,       // source location
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CallExpr<'a> {
    pub func: Box<Expr<'a>>,    // function
    pub args: Vec<Expr<'a>>,    // arguments
    pub src:  Source<'a>,       // source location
}

impl<'a> DisplayWith<AsmFlavor> for Expr<'a> {
    fn fmt(&self, f: &mut Formatter, a: &AsmFlavor) -> fmt::Result {
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Terminals

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Id<'a> {
    pub name: &'a str,
    pub src:  Source<'a>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StrLit<'a> {
    pub val: &'a str,
    pub src: Source<'a>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct IntLit<'a> {
    pub val: BigInt,
    pub src: Source<'a>,
}

impl<'a> Default for Id<'a> {
    #[inline]
    fn default() -> Self {
        Id { name: "", src: Source::BuiltIn }
    }
}

// -----------------------------------------------------------------------------

//impl<T: Target> Display for Expr<T> {
//    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//        match *self {
//            Expr::Ident  (_, s)                      => f.write_str(s.as_ref()),
//            Expr::Str    (_, s)                      => fmt_str(s.as_ref(), f),
//            Expr::Int    (_, ref i)                  => fmt_int(i, f),
//            Expr::Unary  (_, ref o, s, ref x)        => panic!(),
//            Expr::Binary (_, ref o, s, ref x, ref y) => panic!(),
//            _                                        => Err(fmt::Error)
//        }
//    }
//}
//
//fn fmt_str(s: &str, f: &mut Formatter) -> fmt::Result {
//    try!(f.write_char('"'));
//    for c in s.chars() {
//        match c {
//            '\x08'          => try!(f.write_str("\\b")),
//            '\x09'          => try!(f.write_str("\\t")),
//            '\x0A'          => try!(f.write_str("\\n")),
//            '\x0C'          => try!(f.write_str("\\f")),
//            '\x0D'          => try!(f.write_str("\\r")),
//            '\"'            => try!(f.write_str("\\\"")),
//            '\\'            => try!(f.write_str("\\\\")),
//            '\x20'...'\x7E' => try!(f.write_char(c)),
//            _               => try!(fmt_esc_utf8(c, f))
//        }
//    }
//    try!(f.write_char('"'));
//    Ok(())
//}
//
//fn fmt_esc_utf8(c: char, f: &mut Formatter) -> fmt::Result {
//    use std::io::{Cursor, Write};
//    let mut buf = [0u8; 4];
//    let len = {
//        let mut cur = Cursor::new(&mut buf[..]);
//        write!(cur, "{}", c).unwrap();
//        cur.position() as usize
//    };
//    for b in &buf[0..len] {
//        try!(write!(f, "\\{:03o}", b));
//    }
//    Ok(())
//}
//
//fn fmt_int(i: &BigInt, f: &mut Formatter) -> fmt::Result {
//    match i.to_u64() {
//        Some(n) if n > 9 => write!(f, "{:#X}", n),
//        _                => write!(f, "{}",    i),
//    }
//}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use aex::source::Source::BuiltIn;

//    use aex::target::tests::TestTarget;
//    use aex::pos::tests::BOF;

    #[test]
    fn id_default() {
        assert_eq!(
            Id::default(),
            Id { name: "", src: BuiltIn }
        );
    }

//    #[test]
//    fn fmt_ident() {
//        let expr = Expr::Ident::<TestTarget>(BOF, "a");
//        let text = format!("{}", &expr);
//        assert_eq!(text, "a");
//    }

//    #[test]
//    fn fmt_str() {
//        let original = "\
//            \x08\x09\x0A\x0C\x0D\
//            !\"#$%&'()*+,-./0123456789:;<=>?\
//            @ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_\
//            `abcdefghijklmnopqrstuvwxyz{|}~\
//            \x13\x7F\u{7FFF}\
//        ";
//        let formatted = "\"\
//            \\b\\t\\n\\f\\r\
//            !\\\"#$%&'()*+,-./0123456789:;<=>?\
//            @ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\\\]^_\
//            `abcdefghijklmnopqrstuvwxyz{|}~\
//            \\023\\177\\347\\277\\277\
//        \"";
//
//        let expr = Expr::Str::<TestTarget>(BOF, original);
//        let text = format!("{}", &expr);
//        assert_eq!(text, formatted);
//    }

//    #[test]
//    fn fmt_int_small() {
//        let expr = Expr::Int::<TestTarget>(BOF, 7.into());
//        let text = format!("{}", &expr);
//        assert_eq!(text, "7");
//    }

//    #[test]
//    fn fmt_int_large() {
//        let expr = Expr::Int::<TestTarget>(BOF, 42.into());
//        let text = format!("{}", &expr);
//        assert_eq!(text, "0x2A");
//    }

//    //#[test]
//    //fn fmt_add() {
//    //    let a    = Box::new(Expr::Ident::<TestTarget>(BOF, "a"));
//    //    let b    = Box::new(Expr::Ident::<TestTarget>(BOF, "b"));
//    //    let expr = Expr::Add(a, b, None);
//    //    let text = format!("{}", &expr);
//    //    assert_eq!(text, "(a + b)");
//    //}
}

