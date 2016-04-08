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

use std::fmt::{self, Display, Formatter, Write};
use num::{BigInt, ToPrimitive};

use aex::operator::Op;
use aex::pos::*;
use aex::types::*;

pub type Ast<'a> = Vec<Box<Stmt<'a>>>;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Stmt<'a> {
    // Composite
    Block   (Pos<'a>, Vec<Box<Stmt<'a>>>),

    // Declaration
    TypeDef (Pos<'a>, &'a str, Box<Type<'a>>),
    Label   (Pos<'a>, &'a str),
    Bss     (Pos<'a>, &'a str, Box<Type<'a>>),
    Data    (Pos<'a>, &'a str, Box<Type<'a>>, Box<Expr<'a>>),
    Alias   (Pos<'a>, &'a str, Box<Type<'a>>, Box<Expr<'a>>),
    Func    (Pos<'a>, &'a str, Box<Type<'a>>, Box<Stmt<'a>>),

    // Execution
    Eval    (Pos<'a>, Box<Expr<'a>>),
    Loop    (Pos<'a>, Box<Stmt<'a>>),
    If      (Pos<'a>, Cond<'a>, Box<Stmt<'a>>, Option<Box<Stmt<'a>>>),
    While   (Pos<'a>, Cond<'a>, Box<Stmt<'a>>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Expr<'a, T=()> {
    // Atoms
    Ident      (&'a str),
    Str        (&'a str),
    Int        (&'a Pos<'a>, BigInt),
    Deref      (Vec<Box<Expr<'a>>>),

    // Composites
    UnaryOp    (T, &'a Op, &'a str, Box<Expr<'a, T>>),
    BinaryOp   (T, &'a Op, &'a str, Box<Expr<'a, T>>, Box<Expr<'a, T>>),

    // Right Unary
    Member     (Box<Expr<'a>>, &'a str),

    // Left Unary
    Increment  (Box<Expr<'a>>, Option<&'a str>),
    Decrement  (Box<Expr<'a>>, Option<&'a str>),
    Clear      (Box<Expr<'a>>, Option<&'a str>),
    Negate     (Box<Expr<'a>>, Option<&'a str>),
    Complement (Box<Expr<'a>>, Option<&'a str>),

    // Multiplicative
    Multiply   (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    Divide     (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    Modulo     (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),

    // Additive
    Add        (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    Subtract   (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),

    // Bitwise Shift
    ShiftL     (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    ShiftR     (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),

    // Bitwise Boolean
    BitAnd     (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    BitXor     (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    BitOr      (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),

    // Bit Manipulation
    BitChange  (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    BitClear   (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    BitSet     (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    BitTest    (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),

    // Comparison
    Compare    (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    CompareEq  (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    CompareNe  (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    CompareLt  (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    CompareLe  (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    CompareGt  (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    CompareGe  (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    Test       (Box<Expr<'a>>,                Option<&'a str>),

    // Assignment
    Move       (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    MoveCond   (Box<Expr<'a>>, Box<Cond<'a>>, Option<&'a str>), // TODO: Make Cond an atom
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Cond<'a> (pub &'a str, pub Option<Box<Expr<'a>>>);

impl<'a> Expr<'a> {
    pub fn add<'b>(a: Expr<'b>, b: Expr<'b>) -> Expr<'b> {
        Expr::Add(Box::new(a), Box::new(b), None)
    }
}

impl<'a> Display for Expr<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Expr::Ident      (s)                  => f.write_str(s),
            Expr::Str        (s)                  => fmt_str(s, f),
            Expr::Int        (_, ref i)           => fmt_int(i, f),
            Expr::Negate     (ref e, None)        => write!(f, "-{}", e),
            Expr::Complement (ref e, None)        => write!(f, "~{}", e),
            Expr::Multiply   (ref l, ref r, None) => write!(f, "({} * {})",  l, r),
            Expr::Divide     (ref l, ref r, None) => write!(f, "({} / {})",  l, r),
            Expr::Modulo     (ref l, ref r, None) => write!(f, "({} % {})",  l, r),
            Expr::Add        (ref l, ref r, None) => write!(f, "({} + {})",  l, r),
            Expr::Subtract   (ref l, ref r, None) => write!(f, "({} - {})",  l, r),
            Expr::ShiftL     (ref l, ref r, None) => write!(f, "({} << {})", l, r),
            Expr::ShiftR     (ref l, ref r, None) => write!(f, "({} >> {})", l, r),
            Expr::BitAnd     (ref l, ref r, None) => write!(f, "({} & {})",  l, r),
            Expr::BitXor     (ref l, ref r, None) => write!(f, "({} ^ {})",  l, r),
            Expr::BitOr      (ref l, ref r, None) => write!(f, "({} | {})",  l, r),
            _                                     => Err(fmt::Error)
        }
    }
}

fn fmt_str(s: &str, f: &mut Formatter) -> fmt::Result {
    try!(f.write_char('"'));
    for c in s.chars() {
        match c {
            '\x08'          => try!(f.write_str("\\b")),
            '\x09'          => try!(f.write_str("\\t")),
            '\x0A'          => try!(f.write_str("\\n")),
            '\x0C'          => try!(f.write_str("\\f")),
            '\x0D'          => try!(f.write_str("\\r")),
            '\"'            => try!(f.write_str("\\\"")),
            '\\'            => try!(f.write_str("\\\\")),
            '\x20'...'\x7E' => try!(f.write_char(c)),
            _               => try!(fmt_esc_utf8(c, f))
        }
    }
    try!(f.write_char('"'));
    Ok(())
}

fn fmt_esc_utf8(c: char, f: &mut Formatter) -> fmt::Result {
    use std::io::{Cursor, Write};
    let mut buf = [0u8; 4];
    let len = {
        let mut cur = Cursor::new(&mut buf[..]);
        write!(cur, "{}", c).unwrap();
        cur.position() as usize
    };
    for b in &buf[0..len] {
        try!(write!(f, "\\{:03o}", b));
    }
    Ok(())
}

fn fmt_int(i: &BigInt, f: &mut Formatter) -> fmt::Result {
    match i.to_u64() {
        Some(n) if n > 9 => write!(f, "{:#X}", n),
        _                => write!(f, "{}",    i),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aex::pos::Pos;

    #[test]
    fn fmt_ident() {
        let expr = Expr::Ident("a");
        let text = format!("{}", &expr);
        assert_eq!(text, "a");
    }

    #[test]
    fn fmt_str() {
        let original = "\
            \x08\x09\x0A\x0C\x0D\
            !\"#$%&'()*+,-./0123456789:;<=>?\
            @ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_\
            `abcdefghijklmnopqrstuvwxyz{|}~\
            \x13\x7F\u{7FFF}\
        ";
        let formatted = "\"\
            \\b\\t\\n\\f\\r\
            !\\\"#$%&'()*+,-./0123456789:;<=>?\
            @ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\\\]^_\
            `abcdefghijklmnopqrstuvwxyz{|}~\
            \\023\\177\\347\\277\\277\
        \"";

        let expr = Expr::Str(original);
        let text = format!("{}", &expr);
        assert_eq!(text, formatted);
    }

    #[test]
    fn fmt_int_small() {
        let pos  = Pos::bof("f");
        let expr = Expr::Int(&pos, 7.into());
        let text = format!("{}", &expr);
        assert_eq!(text, "7");
    }

    #[test]
    fn fmt_int_large() {
        let pos  = Pos::bof("f");
        let expr = Expr::Int(&pos, 42.into());
        let text = format!("{}", &expr);
        assert_eq!(text, "0x2A");
    }

    #[test]
    fn fmt_add() {
        let a    = Box::new(Expr::Ident("a"));
        let b    = Box::new(Expr::Ident("b"));
        let expr = Expr::Add(a, b, None);
        let text = format!("{}", &expr);
        assert_eq!(text, "(a + b)");
    }
}

