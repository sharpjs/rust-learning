// Abstract Syntax Tree
//
// This file is part of AEx.
// Copyright (C) 2015 Jeffrey Sharp
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

use num::BigInt;
use util::*;
pub use types::Type;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Node<'a> {
    Stmt (Stmt<'a>, Pos),
    Expr (Expr<'a>, Pos),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Stmt<'a> {
    // Meta
    Block   (Vec<Stmt<'a>>),

    // Declaration
    TypeDef (&'a str, Box<Type<'a>>),
    Label   (&'a str),
    Bss     (&'a str, Box<Type<'a>>),
    Data    (&'a str, Box<Type<'a>>, Box<Expr<'a>>),
    Alias   (&'a str, Box<Type<'a>>, Box<Expr<'a>>),
    Func    (&'a str, Box<Type<'a>>, Box<Stmt<'a>>),

    // Execution
    Eval    (Box<Expr<'a>>),
    Loop    (Box<Stmt<'a>>),
    If      (Cond<'a>, Box<Stmt<'a>>, Option<Box<Stmt<'a>>>),
    While   (Cond<'a>, Box<Stmt<'a>>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Expr<'a> {
    Ident      (&'a str),
    Str        (&'a str),
    Int        (BigInt),

    MemberOf   (Box<Expr<'a>>, &'a str),

    Increment  (Box<Expr<'a>>, Option<&'a str>),
    Decrement  (Box<Expr<'a>>, Option<&'a str>),

    Clear      (Box<Expr<'a>>, Option<&'a str>),
    Negate     (Box<Expr<'a>>, Option<&'a str>),
    Complement (Box<Expr<'a>>, Option<&'a str>),

    Multiply   (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    Divide     (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    Modulo     (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),

    Add        (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    Subtract   (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),

    ShiftL     (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    ShiftR     (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),

    BitAnd     (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    BitXor     (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    BitOr      (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),

    BitChange  (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    BitClear   (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    BitSet     (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    BitTest    (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),

    Compare    (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),

    Move       (Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>),
    MoveCond   (Box<Expr<'a>>, Box<Cond<'a>>, Option<&'a str>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Cond<'a> (pub &'a str, pub Option<Box<Expr<'a>>>);

