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
    Id          (Id         <'a>),  // identifier
    Str         (StrLit     <'a>),  // string literal
    Int         (IntLit     <'a>),  // number literal
    // 11: Postfix
    PostInc     (UnaryExpr  <'a>),  // x++      post-increment
    PostDec     (UnaryExpr  <'a>),  // x--      post-decrement
    Call        (CallExpr   <'a>),  // f(x,y)   function call
    Element     (BinaryExpr <'a>),  // x[y]     element (offset?)
    Member      (BinaryExpr <'a>),  // x.y      member  (offset?)
    // 10: Prefix
    PreInc      (UnaryExpr  <'a>),  // ++x      pre-increment
    PreDec      (UnaryExpr  <'a>),  // --x      pre-decrement
    Ref         (UnaryExpr  <'a>),  // &x       address-of
    Deref       (UnaryExpr  <'a>),  // *x       dereference
    Clear       (UnaryExpr  <'a>),  // !x       clear
    Not         (UnaryExpr  <'a>),  // ~x       bitwise not
    Negate      (UnaryExpr  <'a>),  // -x       arithmetic negate
    // 9: Type-Changing
    Cast        (CastExpr   <'a>),  // x:  t    type cast
    Convert     (CastExpr   <'a>),  // x:> t    type conversion
    // 8: Multiplicative
    Multiply    (BinaryExpr <'a>),  // x * y    multiply
    Divide      (BinaryExpr <'a>),  // x / y    divide
    Modulo      (BinaryExpr <'a>),  // x % y    modulo
    // 7: Additive
    Add         (BinaryExpr <'a>),  // x + y    add
    Subtract    (BinaryExpr <'a>),  // x - y    subtract
    // 6: Shift
    ShiftL      (BinaryExpr <'a>),  // x <<  y  shift  left
    ShiftR      (BinaryExpr <'a>),  // x >>  y  shift  right
    RotateL     (BinaryExpr <'a>),  // x <<| y  rotate left
    RotateR     (BinaryExpr <'a>),  // x |>> y  rotate right
    RotateCL    (BinaryExpr <'a>),  // x <<+ y  rotate left  through carry
    RotateCR    (BinaryExpr <'a>),  // x +>> y  rotate right through carry
    // 5: And
    And         (BinaryExpr <'a>),  // x & y    bitwise and
    // 4: Xor
    Xor         (BinaryExpr <'a>),  // x ^ y    bitwise xor
    // 3: Or
    Or          (BinaryExpr <'a>),  // x | y    bitwise or
    // 2: Comparison
    Compare     (BinaryExpr <'a>),  // x <> y   compare
    Test        (UnaryExpr  <'a>),  // x?       test (compare with zero)
    // 1: Conditional
    Equal       (BinaryExpr <'a>),  // x == y   equal
    NotEqual    (BinaryExpr <'a>),  // x != y   not equal
    Less        (BinaryExpr <'a>),  // x <  y   less than
    LessEqual   (BinaryExpr <'a>),  // x <= y   less than or equal
    More        (BinaryExpr <'a>),  // x >  y   more than
    MoreEqual   (BinaryExpr <'a>),  // x >= y   more than or equal
  //Yields      (YieldsExpr <'a>),  // x => c   yields-condition
    // 0: Assignment
    Assign      (BinaryExpr <'a>),  // x = y    assignment

    // Obsolete
    Unary       (UnaryExpr  <'a>),  // GET RID OF THIS
    Binary      (BinaryExpr <'a>),  // GET RID OF THIS
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

impl<'a> Expr<'a> {
    pub fn src(&self) -> Source<'a> {
        Source::BuiltIn // TODO
    }
}

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

