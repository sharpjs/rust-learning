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
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            //target:  target,
            strings:   StringInterner::new(),
        }
    }

    pub fn compile(mut self, name: &str, input: &str) {
        let out = &mut Output::new();

        // Step 1
        let mut ast = self.parse(name, input, out);

        // Step 2
        self.check_types(&mut ast, out);

        // Step 3
        self.generate_code(&ast, out);

        // Why can't we do type checking during code generation?
        // There was a reason, but I forget.

        // Step 4
        // Do something with output

        println!("{:#?}", ast);
    }

    fn parse<'a>(&'a self,
                 input: &'a str,
                 name:  &'a str,
                 out:   &mut Output<'a>
                ) -> Ast<'a> {
        let lexer = Lexer::new(self, input.chars(), name);
        parse(lexer)
    }

    fn check_types<'a>(&'a self,
                       ast: &mut Ast<'a>,
                       out: &mut Output<'a>) {
        // todo
    }

    fn generate_code<'a>(&'a self,
                         ast: &Ast<'a>,
                         out: &mut Output<'a>) {
        // todo
        // let generator = CodeGenerator::new(&mut compilation);
    }
}

// Type Declaration Pass:
//
// create queue for forward resolutions
//
// for each typedef n = t {
//   create hash of forbidden types
//   add t to hash
//
//   t @ Ident n'     // look up t' by n'
//                    //  - if none, enqueue
//                    //  - if t' forbidden, error
//                    //  - else, ok (t' already checked)
//   t @ Int          // ok
//   t @ Float        // ok
//   t @ Array t'     // recurse for t'
//   t @ Ptr p v      // recurse for p, v (forbidden v is ok!)
//   t @ Struct       // recurse for members
//   t @ Union        // recurse for members
//   t @ Func         // recurse for members
//   m @ Member n t'  // recurse for t'
//   ... check forbidden t' before recurse (except ptr v)
// }
//
// consume each item in queue {
//   t @ Ident n'   // look up t' by n'
//                  // - if none, enqueue
//                  // - if t' forbidden, error
//                  // else, ok (t' already checked)
// }
//
// if queue size didn't decrease, we have undefined types
//
// Q. Can typedefs include types from other scopes?  A. No

// -----------------------------------------------------------------------------
// STUBS
// -----------------------------------------------------------------------------

use std::marker::PhantomData;

// -----------------------------------------------------------------------------

pub struct Output<'a> (PhantomData<&'a ()>);

impl<'a> Output<'a> { pub fn new() -> Self { Output(PhantomData) } }

// -----------------------------------------------------------------------------

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
struct Ast<'a> (PhantomData<&'a ()>);

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
struct Token<'a> (PhantomData<&'a ()>);

// -----------------------------------------------------------------------------

struct Lexer<'a, I: Iterator<Item=char>> (&'a Compiler, I);

impl<'a, I> Lexer<'a, I> where I: Iterator<Item=char> {
    pub fn new(compiler: &'a Compiler, input: I, name: &'a str) -> Self {
        Lexer(compiler, input)
    }
}

trait Lex<'a> {
    fn lex(&mut self) -> Token<'a>;
}

impl<'a, I> Lex<'a> for Lexer<'a, I> where I: Iterator<Item=char> {
    fn lex(&mut self) -> Token<'a> {
        self.1.next();
        self.1.next();
        Token(PhantomData)
    }
}

// -----------------------------------------------------------------------------

fn parse<'a, L>(mut lexer: L) -> Ast<'a> where L: Lex<'a> {
    lexer.lex();
    lexer.lex();
    Ast(PhantomData)
}

