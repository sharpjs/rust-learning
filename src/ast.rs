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

#[derive(Clone, Debug)]
pub enum Stmt {
    Block   (Vec<Stmt>),
    Eval    (Box<Expr>),
}

// data Stmt
//     = Block     [Stmt]
//     | TypeDef   String Type
//     | Label     String
//     | Bss       String Type
//     | Data      String Type Exp
//     | Alias     String Type Exp
//     | Func      String Type Stmt
//     | Eval      Exp
//     | Loop      Stmt
//     | If        Test Stmt Stmt
//     | While     Test Stmt
//     deriving (Eq, Show)

#[derive(Clone, Debug)]
pub enum Expr {
    Ident   (StrId),
    Str     (StrId),
    Int     (u64),

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
    SetCond    (Box<Expr>, Box<Test>, Option<StrId>),
}

#[derive(Clone, Debug)]
pub struct Test (pub StrId, pub Option<Box<Expr>>);

