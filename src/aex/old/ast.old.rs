// Abstract Syntax Tree
//
// This file is part of AEx.
// Copyright (C) 2017 Jeffrey Sharp
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

//use aex::pos::Source;
use aex::target::Target;
//use aex::types::Type;

pub type Ast<T> = Vec<Stmt<T>>;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Stmt<T: Target> {
    // Composite
    Block   (T::Source, Vec<Stmt<T>>),

    // Declaration
    //TypeDef (T::Source, T::String, Box<Type<T>>),
    Label   (T::Source, T::String),
    //Bss     (T::Source, T::String, Box<Type<T>>),
    //Data    (T::Source, T::String, Box<Type<T>>, Box<Expr<T>>),
    //Alias   (T::Source, T::String, Box<Type<T>>, Box<Expr<T>>),
    //Func    (T::Source, T::String, Box<Type<T>>, Box<Stmt<T>>),

    // Execution
    Eval    (T::Source, Box<Expr<T>>),
    Loop    (T::Source, Box<Stmt<T>>),
    If      (T::Source, Cond<T>, Box<Stmt<T>>, Option<Box<Stmt<T>>>),
    While   (T::Source, Cond<T>, Box<Stmt<T>>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Expr<T: Target> {
    // Atomic
    Ident   (T::Source, T::String),
    Str     (T::Source, T::String),
    Int     (T::Source, BigInt),
    Deref   (T::Source, Vec<Expr<T>>),
    Member  (T::Source, Box<Expr<T>>, T::String),

    // Composite
    Unary   (T::Source, T::Operator, T::String, Box<Expr<T>>),
    Binary  (T::Source, T::Operator, T::String, Box<Expr<T>>, Box<Expr<T>>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Cond<T: Target> (T::String, Option<Box<Expr<T>>>);

impl<T: Target> Display for Expr<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Expr::Ident  (_, s)                      => f.write_str(s.as_ref()),
            Expr::Str    (_, s)                      => fmt_str(s.as_ref(), f),
            Expr::Int    (_, ref i)                  => fmt_int(i, f),
            Expr::Unary  (_, ref o, s, ref x)        => panic!(),
            Expr::Binary (_, ref o, s, ref x, ref y) => panic!(),
            _                                        => Err(fmt::Error)
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

    use aex::target::tests::TestTarget;
    use aex::pos::tests::BOF;

    #[test]
    fn fmt_ident() {
        let expr = Expr::Ident::<TestTarget>(BOF, "a");
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

        let expr = Expr::Str::<TestTarget>(BOF, original);
        let text = format!("{}", &expr);
        assert_eq!(text, formatted);
    }

    #[test]
    fn fmt_int_small() {
        let expr = Expr::Int::<TestTarget>(BOF, 7.into());
        let text = format!("{}", &expr);
        assert_eq!(text, "7");
    }

    #[test]
    fn fmt_int_large() {
        let expr = Expr::Int::<TestTarget>(BOF, 42.into());
        let text = format!("{}", &expr);
        assert_eq!(text, "0x2A");
    }

    //#[test]
    //fn fmt_add() {
    //    let a    = Box::new(Expr::Ident::<TestTarget>(BOF, "a"));
    //    let b    = Box::new(Expr::Ident::<TestTarget>(BOF, "b"));
    //    let expr = Expr::Add(a, b, None);
    //    let text = format!("{}", &expr);
    //    assert_eq!(text, "(a + b)");
    //}
}

