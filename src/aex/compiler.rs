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

#![allow(unused_mut)]

//use aex::asm::Assembly;
//use aex::codegen::CodeGenerator;
//use aex::lexer::Lexer;
//use aex::mem::arena::Arena;
use aex::mem::StringInterner;
//use aex::message::Messages;
//use aex::operator::{self, OpTable};
//use aex::parser::parse;
//use aex::pos::Pos;
//use aex::target::Target;

//use aex::target::ColdFire;

pub struct Compiler {
    //pub target:  T,
    pub strings:   StringInterner,
    //pub code:    Assembly,
    //pub log:     Messages<'a>,
    //pub ops:     OpTable,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            //target:  target,
            strings:   StringInterner::new(),
            //code:    Assembly::new(),
            //log:     Messages::new(),
            //ops:     operator::create_op_table()
        }
    }

    pub fn compile<I>(mut self, input: I, filename: &str)
    where I: Iterator<Item=char> {

        // Step 1
        let mut ast = self.parse(input, filename);

        // Step 2
        self.check_types(&mut ast);

        // Step 3
        self.generate_code(&ast);

        // Step 4
        // Do something with output

        println!("{:#?}", ast);
    }

    fn parse<'a, I>(&mut self, input: I, filename: &'a str) -> Ast<'a>
    where I: Iterator<Item=char> {
        Ast(PhantomData)
        //let lexer = Lexer::new(&mut self, input);
        //parse(lexer)
    }

    fn check_types<'a>(&mut self, ast: &mut Ast<'a>) {
    }

    fn generate_code<'a>(&mut self, ast: &Ast<'a>) {
        // let generator = CodeGenerator::new(&mut compilation);
    }
}

// Stubs

use std::marker::PhantomData;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
struct Ast<'a>
    (PhantomData<&'a ()>);

struct Lexer<'a, I: Iterator<Item=char>>
    (&'a mut Compiler, I);

impl<'a, I: Iterator<Item=char>> Lexer<'a, I> {
    pub fn new(compiler: &'a mut Compiler, input: I) -> Self {
        Lexer(compiler, input)
    }
}

fn parse<'a, I: Iterator<Item=char>>(mut lexer: Lexer<'a, I>) -> Ast<'a> {
    Ast(PhantomData)
}

