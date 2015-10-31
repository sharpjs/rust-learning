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

    fn parse(mut self) -> Self {
        self.result.ast = self.parse_stmts();
        self
    }

    // stmts:
    //   EOS? (stmt EOS)<* (stmt EOS?)?
    //
    fn parse_stmts(&mut self) -> Many<Stmt> {
        let s = try!(self.parse_stmt());
        Ok(vec![s])
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
        let (l, token, r) = self.lexer.lex();

        match token {
            Token::Int(x) => Ok(Box::new(Expr::Int(x))),
            _         => Err(())
        }
    }
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use ast::*;
    use ast::Expr::*;

    #[test]
    fn empty() {
        assert_eq!(
            parse("42".chars()).ast,
            Ok(vec![Box::new(Stmt::Eval(Box::new(Int(42))))])
        );
    }
}

