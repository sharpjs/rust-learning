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
use aex::operator::{BinaryOperator, UnaryOperator};

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
    // Atomic
    Id      (Id         <'a>),  // x
    Str     (StrLit     <'a>),  // "hello"
    Int     (IntLit     <'a>),  // 42

    // Atomic-ish?
    Deref   (DerefExpr  <'a>),  // [a0, 8, d0*4]
    Member  (MemberExpr <'a>),  // x.y

    // Composite
    Unary   (UnaryExpr  <'a>),  // ~x
    Binary  (BinaryExpr <'a>),  // x + y
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DerefExpr<'a> {
    pub expr: Vec<Expr<'a>>,
    pub src:  Source<'a>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MemberExpr<'a> {
    pub expr: Box<Expr<'a>>,
    pub id:   Id<'a>,
    pub src:  Source<'a>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UnaryExpr<'a> {
    pub op:   &'a UnaryOperator,
    pub sel:  Id<'a>,
    pub expr: Box<Expr<'a>>,
    pub src:  Source<'a>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BinaryExpr<'a> {
    pub op:   &'a BinaryOperator,
    pub sel:  Id<'a>,
    pub l:    Box<Expr<'a>>,
    pub r:    Box<Expr<'a>>,
    pub src:  Source<'a>,
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

