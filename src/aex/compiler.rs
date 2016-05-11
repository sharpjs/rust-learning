// Compiler Top-Level Interface
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

//use aex::asm::Assembly;
//use aex::codegen::CodeGenerator;
//use aex::lexer::Lexer;
//use aex::mem::arena::Arena;
//use aex::mem::interner::StringInterner;
//use aex::message::Messages;
//use aex::operator::{self, OpTable};
//use aex::parser::parse;
//use aex::pos::Pos;
//use aex::target::Target;

//use aex::target::ColdFire;

pub fn compile<I>(input: I, filename: &str)
where I: Iterator<Item=char> {
//    let target   = ColdFire::new();
//    let memory   = Memory::new();
//    let compiler = Compiler::new(target, &memory);

//    let ast = {
//        let mut lexer = Lexer::new(&mut compilation, input);
//        parse(&mut lexer)
//    };
//
//    let generator = CodeGenerator::new(&mut compilation);
//
//    println!("{:#?}", ast);
}

//struct Compiler<'a> {
//    pub target:    T,
//    pub strings:   &'a StringInterner<'a>,
//    pub positions: &'a Arena<Pos<'a>>,
    //pub code:      Assembly,
    //pub log:       Messages<'a>,
    //pub ops:       OpTable,
//}

//impl<'a, T: Target> Compiler<'a, T> {
//    pub fn new(target: T, memory: &'a Memory<'a>) -> Self {
//        Compiler {
//            target:    target,
//            strings:   &memory.strings,
//            positions: &memory.positions,
//            code:      Assembly::new(),
//            log:       Messages::new(),
//            ops:       operator::create_op_table()
//        }
//    }
//}

// This type is separate, so that rustc generates a useful 'a.
//
//struct Memory<'a> {
//    strings:   StringInterner<'a>,
//    positions: Arena<Pos<'a>>,
//}
//
//impl<'a> Memory<'a> {
//    pub fn new() -> Self {
//        Memory {
//            strings:   StringInterner::new(),
//            positions: Arena::new(),
//        }
//    }
//}

