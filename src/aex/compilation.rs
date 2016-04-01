// Compilation State
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

use aex::asm::Assembly;
use aex::lexer::Lexer;
use aex::mem::interner::StringInterner;
use aex::message::Messages;
use aex::operator::{self, OpTable};
use aex::parser::parse;

pub fn compile<I>(input: I, filename: &str)
where I: Iterator<Item=char> {
    let mut compilation = Compilation::new();
    let     lexer       = Lexer::new(&mut compilation, input);
    let ast = parse(lexer);

    println!("{:#?}", ast);
}

pub struct Compilation<'a> {
    pub strings: StringInterner<'a>,
    pub code:    Assembly,
    pub log:     Messages<'a>,
    pub ops:     OpTable,
    // target
    //  > builtin scope
    //  > operators
}

impl<'a> Compilation<'a> {
    pub fn new() -> Self {
        Compilation {
            strings: StringInterner::new(),
            code:    Assembly::new(),
            log:     Messages::new(),
            ops:     operator::create_op_table()
        }
    }
}

