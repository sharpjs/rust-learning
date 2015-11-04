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

use interner::StrId;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Stmt {
    // Meta
    Block   (Vec<Stmt>),
    // Declarational
    TypeDef (StrId, Box<Type>),
    Label   (StrId),
    Bss     (StrId, Box<Type>),
    Data    (StrId, Box<Type>, Box<Expr>),
    Alias   (StrId, Box<Type>, Box<Expr>),
    Func    (StrId, Box<Type>, Box<Stmt>),
    // Executable
    Eval    (Box<Expr>),
    Loop    (Box<Stmt>),
    If      (Cond, Box<Stmt>, Option<Box<Stmt>>),
    While   (Cond, Box<Stmt>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Type {
    TypeRef (StrId),
    Array   (Box<Type>, Option<u64>),
    Ptr     (Box<Type>, Box<Type>),
    Struct  (Vec<Member>),
    Union   (Vec<Member>),
    Func    (Vec<Member>, Vec<Member>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Member (StrId, Box<Type>);

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Expr {
    Ident      (StrId),
    Str        (StrId),
    Int        (u64),

    MemberOf   (Box<Expr>, StrId),
    Increment  (Box<Expr>, Option<StrId>),
    Decrement  (Box<Expr>, Option<StrId>),

    Clear      (Box<Expr>, Option<StrId>),
    Negate     (Box<Expr>, Option<StrId>),
    Complement (Box<Expr>, Option<StrId>),

    Multiply   (Box<Expr>, Box<Expr>, Option<StrId>),
    Divide     (Box<Expr>, Box<Expr>, Option<StrId>),
    Modulo     (Box<Expr>, Box<Expr>, Option<StrId>),

    Add        (Box<Expr>, Box<Expr>, Option<StrId>),
    Subtract   (Box<Expr>, Box<Expr>, Option<StrId>),

    ShiftL     (Box<Expr>, Box<Expr>, Option<StrId>),
    ShiftR     (Box<Expr>, Box<Expr>, Option<StrId>),

    BitAnd     (Box<Expr>, Box<Expr>, Option<StrId>),
    BitXor     (Box<Expr>, Box<Expr>, Option<StrId>),
    BitOr      (Box<Expr>, Box<Expr>, Option<StrId>),

    BitChange  (Box<Expr>, Box<Expr>, Option<StrId>),
    BitClear   (Box<Expr>, Box<Expr>, Option<StrId>),
    BitSet     (Box<Expr>, Box<Expr>, Option<StrId>),
    BitTest    (Box<Expr>, Box<Expr>, Option<StrId>),

    Compare    (Box<Expr>, Box<Expr>, Option<StrId>),

    Set        (Box<Expr>, Box<Expr>, Option<StrId>),
    SetCond    (Box<Expr>, Box<Cond>, Option<StrId>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Cond (pub StrId, pub Option<Box<Expr>>);

