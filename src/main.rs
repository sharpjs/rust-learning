// AEx - Just a toy language for learning Rust
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

#![allow(dead_code)]
#![allow(unused_variables)]

macro_rules! is {
    { $val:expr => $( $pat:pat ),* } => {
        match $val {
            $( $pat => true ),* ,
            _ => false
        }
    };
    { $val:expr => $( $pat:pat if $cond:expr ),* } => {
        match $val {
            $( $pat if $cond => true ),* ,
            _ => false
        }
    };
}

mod ast;
mod interner;
mod lexer;
mod message;
mod parser;

// TODO: Move following into parser module

use ast::*;
use interner::*;
use lexer::*;
use message::*;
use lalrpop_util::*;
extern crate lalrpop_util;

use std::borrow::Borrow;
use std::rc::Rc;

pub type Error = ParseError<Pos, Token, (Pos, Message)>;

pub fn parse<S: AsRef<str>>(s: S) -> Result<Vec<Stmt>, Error> {
    let chars   = s.as_ref().chars();
    let strings = Rc::new(Interner::new());
    let lexer   = Lexer::new(strings.clone(), chars);
    parser::parse_Stmts(strings.borrow(), lexer)
}

fn main() {
    let x = parse("a++:q * -3 + (m >> 2) & 4 | 3 ^ 5 / what; b.c \n c--");
    println!("\n{:#?}", x);
}

#[test]
fn test_main() {
    main()
}

