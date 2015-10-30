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
use interner::Interner;
use lexer::{Lexer, Token};
use std::rc::Rc;
//use message::*;

pub fn parser<I: Iterator<Item=char>>(input: I) -> Parser<I> {
    let strings = Rc::new(Interner::new());
    let lexer   = Box::new(Lexer::new(strings.clone(), input));
    Parser {
        lexer: lexer,
        strings: strings
    }
}

pub struct Parser<I: Iterator<Item=char>> {
    lexer:   Box<Lexer<I>>,
    strings: Rc<Interner>,
}

impl<I: Iterator<Item=char>> Parser<I> {
    pub fn parse_expr(&mut self) -> Option<Box<Expr>> {
        let (l, token, r) = self.lexer.lex();

        match token {
            Token::Int(x) => Some(Box::new(Expr::Int(x))),
            _ => None
        }
    }
}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use ast::*;
    use ast::Expr::*;

    #[test]
    fn empty() {
        parse("42").yields_expr(Int(42));
    }

    // Test Harness

    use super::*;
    use std::str::Chars;

    struct ParserHarness<'a> (Parser<Chars<'a>>);

    fn parse(input: &str) -> ParserHarness {
        let chars   = input.chars();
        let parser  = parser(chars);
        ParserHarness(parser)
    }

    impl<'a> ParserHarness<'a> {
        fn yields_expr(&mut self, expr: Expr) -> &mut Self {
            assert_eq!(expr, *self.0.parse_expr().unwrap());
            self
        }
    }
}

