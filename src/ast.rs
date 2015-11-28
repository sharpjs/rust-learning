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

use std::rc::Rc;
use num::BigInt;
pub use types::*;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Stmt {
    // Meta
    Block   (Vec<Stmt>),
    // Declaration
    TypeDef (Rc<String>, Box<Type>),
    Label   (Rc<String>),
    Bss     (Rc<String>, Box<Type>),
    Data    (Rc<String>, Box<Type>, Box<Expr>),
    Alias   (Rc<String>, Box<Type>, Box<Expr>),
    Func    (Rc<String>, Box<Type>, Box<Stmt>),
    // Execution
    Eval    (Box<Expr>),
    Loop    (Box<Stmt>),
    If      (Cond, Box<Stmt>, Option<Box<Stmt>>),
    While   (Cond, Box<Stmt>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Expr {
    Ident      (Rc<String>),
    Str        (Rc<String>),
    Int        (BigInt),

    MemberOf   (Box<Expr>, Rc<String>),
    Increment  (Box<Expr>, Option<Rc<String>>),
    Decrement  (Box<Expr>, Option<Rc<String>>),

    Clear      (Box<Expr>, Option<Rc<String>>),
    Negate     (Box<Expr>, Option<Rc<String>>),
    Complement (Box<Expr>, Option<Rc<String>>),

    Multiply   (Box<Expr>, Box<Expr>, Option<Rc<String>>),
    Divide     (Box<Expr>, Box<Expr>, Option<Rc<String>>),
    Modulo     (Box<Expr>, Box<Expr>, Option<Rc<String>>),

    Add        (Box<Expr>, Box<Expr>, Option<Rc<String>>),
    Subtract   (Box<Expr>, Box<Expr>, Option<Rc<String>>),

    ShiftL     (Box<Expr>, Box<Expr>, Option<Rc<String>>),
    ShiftR     (Box<Expr>, Box<Expr>, Option<Rc<String>>),

    BitAnd     (Box<Expr>, Box<Expr>, Option<Rc<String>>),
    BitXor     (Box<Expr>, Box<Expr>, Option<Rc<String>>),
    BitOr      (Box<Expr>, Box<Expr>, Option<Rc<String>>),

    BitChange  (Box<Expr>, Box<Expr>, Option<Rc<String>>),
    BitClear   (Box<Expr>, Box<Expr>, Option<Rc<String>>),
    BitSet     (Box<Expr>, Box<Expr>, Option<Rc<String>>),
    BitTest    (Box<Expr>, Box<Expr>, Option<Rc<String>>),

    Compare    (Box<Expr>, Box<Expr>, Option<Rc<String>>),

    Set        (Box<Expr>, Box<Expr>, Option<Rc<String>>),
    SetCond    (Box<Expr>, Box<Cond>, Option<Rc<String>>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Cond (pub Rc<String>, pub Option<Box<Expr>>);

