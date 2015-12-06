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

// Returns true if the value matches any of the given patterns.
//
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

extern crate num;

#[macro_use]
mod util;

//mod analyze;
mod arena;
mod ast;
mod interner;
//mod lexer;
//mod message;
//mod parser;
mod scope;
mod symbol;
mod types;

//mod mcf5307;

fn main() {}

