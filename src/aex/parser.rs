// Parser
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

//use std::rc::Rc;
//use num::bigint::ToBigInt;

//use ast::*;
//use ast::Stmt::*;
//use ast::Expr::*;
//use interner::Interner;
//use message::*;

use aex::ast::{Stmt};
use aex::lexer::{Lex, Token};
use aex::pos::Pos;
use aex::types::Type;

pub fn parse<'a, L: Lex<'a>>(mut lexer: L) -> Stmt<'a> {
    println!("parse: begin");
    let mut parser = Parser::new(&mut lexer);
    parser.parse_stmt();
    //self.parse_stmts_until(Token::Eof);
    println!("parse: end");
    Stmt::Block(Pos::bof("f"), vec![])
}

struct Parser<'p, 'a: 'p, L: 'p + Lex<'a>> {
    token:  Token<'a>,
    span:   (Pos<'a>, Pos<'a>),
    lexer:  &'p mut L,
}

// Helpers

macro_rules! expected {
    ( $parser:ident, $e:expr ) => {{
        $parser.expected($e);
        return Err(())
    }};
}

type One <T> = Result<    Box<T>,  ()>;
type Many<T> = Result<Vec<Box<T>>, ()>;

impl<'p, 'a: 'p, L: 'p + Lex<'a>> Parser<'p, 'a, L> {
    fn new(lexer: &'p mut L) -> Self {
        let (l, token, r) = lexer.lex();
        Parser {
            token:  token,
            span:   (l, r),
            lexer:  lexer,
        }
    }

    // Primitives

    fn advance(&mut self) {
        println!("parser: advancing");
        let (l, tok, r) = self.lexer.lex();
        self.token = tok;
        self.span  = (l, r);
    }

    fn expected(&mut self, description: &str) {
        println!("parser: ERROR: expected: {}", description);
        //self.compilation.log.err_expected(self.span.0, description);
    }

//    // stmts:
//    //   EOS? ( stmt EOS )* stmt? end
//    //
//    fn parse_stmts_until(&mut self, end: Token) -> Many<Stmt> {
//        let mut stmts = vec![];
//        advance!(self, Token::Eos);
//        loop {
//            if self.token == end { return Ok(stmts); }
//            stmts.push(try!(self.parse_stmt()));
//            if self.token == end { return Ok(stmts); }
//            expect!(self, Token::Eos);
//        }
//    }

    // stmt:
    //   typedef
    //   expr
    //
    fn parse_stmt(&mut self) -> One<Stmt> {
        match self.token {
            Token::KwType => self.parse_typedef(),
            _             => expected!(self, "statement")
        }
    }

    // typedef:
    //   'type' id '=' id  // TODO: needs to be type, not id
    //
    fn parse_typedef(&mut self) -> One<Stmt> {
        let pos = self.span.0;

        match self.token {
            Token::KwType => self.advance(),
            _             => expected!(self, "'type' keyword")
        };

        let name = match self.token {
            Token::Id(n) => { self.advance(); n },
            _            => expected!(self, "identifier")
        };

        match self.token {
            Token::Equal => self.advance(),
            _            => expected!(self, "'='")
        };

        let def = match self.token {
            Token::Id(n) => { self.advance(); n },
            _            => expected!(self, "type")
        };

        match self.token {
            Token::Eos => {},
            _          => expected!(self, "end of statement")
        };
        
        Ok(Box::new(
            Stmt::TypeDef(pos, name, Box::new(Type::Ref(def)))
        ))
    }

//    // expr:
//    //   INT
//    //   '(' expr ')'
//    //
//    pub fn parse_expr(&mut self) -> One<Expr> {
//        match self.token {
//            // INT
//            Token::Int(x) => {
//                self.advance();
//                Ok(Box::new(int(x)))
//            },
//            // '(' expr ')'
//            Token::ParenL => {
//                self.advance();
//                let e = self.parse_expr();
//                expect!(self, Token::ParenR);
//                e
//            },
//            _ => Err(())
//        }
//    }
}

// Convenience

//#[inline]
//fn eval(e: Expr) -> Box<Stmt> {
//    Box::new(Eval(Box::new(e)))
//}

//#[inline]
//fn int(n: u64) -> Expr {
//    Int(n.to_bigint().unwrap())
//}

//// -----------------------------------------------------------------------------
//// Tests
//
//#[cfg(test)]
//mod tests {
//    use super::*;
//    use super::eval;
//    use super::int;
//  //use ast::*;
//  //use ast::Expr::*;
//
//    macro_rules! assert_parse {
//        ( $i:expr, $( $s:expr ),* ) => {{
//            assert_eq!(
//                parse($i.chars()).ast,
//                Ok(vec![$($s),*])
//            );
//        }};
//    }
//
//    // Statements
//    #[test] fn empty()         { assert_parse!( "",                                 ); }
//    #[test] fn eos()           { assert_parse!( ";",                                ); }
//    #[test] fn stmt()          { assert_parse!(  "4",    eval(int(4))               ); }
//    #[test] fn eos_stmt()      { assert_parse!( ";4",    eval(int(4))               ); }
//    #[test] fn stmt_eos()      { assert_parse!(  "4;",   eval(int(4))               ); }
//    #[test] fn eos_stmt_eos()  { assert_parse!( ";4;",   eval(int(4))               ); }
//    #[test] fn stmts()         { assert_parse!(  "4;2",  eval(int(4)), eval(int(2)) ); }
//    #[test] fn eos_stmts()     { assert_parse!( ";4;2",  eval(int(4)), eval(int(2)) ); }
//    #[test] fn stmts_eos()     { assert_parse!(  "4;2;", eval(int(4)), eval(int(2)) ); }
//    #[test] fn eos_stmts_eos() { assert_parse!( ";4;2;", eval(int(4)), eval(int(2)) ); }
//
//    // Atomic Expressions
//    #[test] fn parens() { assert_parse!( "(4)", eval(int(4)) ); }
//}
//
