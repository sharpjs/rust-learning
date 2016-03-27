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
use aex::lexer::{Lexer, Lex};
use aex::mem::interner::StringInterner;
use aex::message::Messages;

pub fn compile<I>(input: I, filename: &str)
where I: Iterator<Item=char> {
    let mut compilation = Compilation::new();
    let mut lexer       = Lexer::new(&mut compilation, input);
    lexer.lex();
}

pub struct Compilation<'a> {
    pub strings: StringInterner<'a>,
    pub code:    Assembly,
    pub log:     Messages<'a>,
}

impl<'a> Compilation<'a> {
    fn new() -> Self {
        Compilation {
            strings: StringInterner::new(),
            code:    Assembly::new(),
            log:     Messages::new(),
        }
    }
}

