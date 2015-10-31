// Parser
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

use ast::*;
use ast::Stmt::*;
use ast::Expr::*;
use interner::Interner;
use lexer::{Lexer, Token, Pos};
use std::rc::Rc;
use message::*;

type T       = Token;
type One <T> = Result<    Box<T>,  ()>;
type Many<T> = Result<Vec<Box<T>>, ()>;

pub struct ParseResult {
    pub ast:      Result<Vec<Box<Stmt>>, ()>, // abstract syntax tree, if parse succeeded
    pub strings:  Rc<Interner>,               // interned strings
    pub messages: Vec<Message>,               // messages (errors, warnings, etc.)
}

pub fn parse<I: Iterator<Item=char>>(input: I) -> ParseResult {
    Parser::new(input).parse().result
}

pub trait Parse {
    fn parse(&self) -> ParseResult;
}

struct Parser<I: Iterator<Item=char>> {
    token:  Token,
    span:   (Pos, Pos),
    lexer:  Lexer<I>,
    result: ParseResult,
}

macro_rules! advance {
    ( $p:ident, $t:pat ) => {
        match $p.token {
            $t => { $p.advance(); true },
            _  => false
        }
    };
}

macro_rules! expect {
    ( $p:ident, $t:pat ) => {
        match $p.token {
            $t => $p.advance(),
            _  => return Err(())
        }
    };
}

impl<I: Iterator<Item=char>> Parser<I> {
    fn new(input: I) -> Self {
        let     strings   = Rc::new(Interner::new());
        let mut lexer     = Lexer::new(strings.clone(), input);
        let (l, token, r) = lexer.lex();
        Parser {
            token:  token,
            span:   (l, r),
            lexer:  lexer,
            result: ParseResult {
                ast:      Err(()),
                strings:  strings,
                messages: vec![]
            }
        }
    }

    fn advance(&mut self) -> &mut Self {
        println!("parser: advancing");
        let (l, tok, r) = self.lexer.lex();
        self.token = tok;
        self.span = (l, r);
        self
    }

    fn parse(mut self) -> Self {
        println!("parse: begin");
        self.result.ast = self.parse_stmts_until(Token::Eof);
        println!("parse: end");
        self
    }

    // stmts:
    //   EOS? ( (stmt EOS)* stmt EOS? )? end
    //
    fn parse_stmts_until(&mut self, end: Token) -> Many<Stmt> {
        let mut stmts = vec![];

        // EOS?
        if self.token == Token::Eos { self.advance(); }

        // end
        if self.token == end { return Ok(stmts); }

        loop {
            // stmt
            stmts.push(try!(self.parse_stmt()));

            // end
            if self.token == end { return Ok(stmts); }

            // EOS
            expect!(self, Token::Eos);

            // end
            if self.token == end { return Ok(stmts); }
        }
    }

    // stmt:
    //   expr
    //
    fn parse_stmt(&mut self) -> One<Stmt> {
        let e = try!(self.parse_expr());
        Ok(Box::new(Eval(e)))
    }

    // expr:
    //   INT
    //
    pub fn parse_expr(&mut self) -> One<Expr> {
        match self.token {
            Token::Int(x) => {
                println!("parser: found integer literal");
                self.advance();
                Ok(Box::new(Int(x)))
            },
            _ => Err(())
        }
    }
}

// Convenience

#[inline]
fn eval(e: Expr) -> Box<Stmt> {
    Box::new(Eval(Box::new(e)))
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use super::eval;
  //use ast::*;
    use ast::Expr::*;

    macro_rules! assert_parse {
        ( $i:expr, $( $s:expr ),* ) => {{
            assert_eq!(
                parse($i.chars()).ast,
                Ok(vec![$($s),*])
            );
        }};
    }

    #[test] fn empty()         { assert_parse!( "",                                 ); }
    #[test] fn eos()           { assert_parse!( ";",                                ); }
    #[test] fn stmt()          { assert_parse!(  "4",    eval(Int(4))               ); }
    #[test] fn eos_stmt()      { assert_parse!( ";4",    eval(Int(4))               ); }
    #[test] fn stmt_eos()      { assert_parse!(  "4;",   eval(Int(4))               ); }
    #[test] fn eos_stmt_eos()  { assert_parse!( ";4;",   eval(Int(4))               ); }
    #[test] fn stmts()         { assert_parse!(  "4;2",  eval(Int(4)), eval(Int(2)) ); }
    #[test] fn eos_stmts()     { assert_parse!( ";4;2",  eval(Int(4)), eval(Int(2)) ); }
    #[test] fn stmts_eos()     { assert_parse!(  "4;2;", eval(Int(4)), eval(Int(2)) ); }
    #[test] fn eos_stmts_eos() { assert_parse!( ";4;2;", eval(Int(4)), eval(Int(2)) ); }
}

