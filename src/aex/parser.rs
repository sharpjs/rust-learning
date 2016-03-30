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

//use ast::*;
//use ast::Stmt::*;
//use ast::Expr::*;
//use interner::Interner;
//use message::*;

use num::BigInt;

use aex::ast::{Ast, Stmt, Expr};
use aex::lexer::{Lex, Token};
use aex::pos::Pos;
use aex::types::Type;

use self::Assoc::*;
use self::Fix::*;

type One <T> = Result<    Box<T>,  ()>;
type Many<T> = Result<Vec<Box<T>>, ()>;

pub fn parse<'a, L: Lex<'a>> (mut lexer: L) -> Result<Ast<'a>, ()> {
    println!("parse: begin");
    let stmts = Parser::new(&mut lexer)
        .parse_stmts_until(Token::Eof, "end of statement or EOF");
    println!("parse: end");
    stmts
}

struct Parser<'p, 'a: 'p, L: 'p + Lex<'a>> {
    token:  Token<'a>,
    span:   (Pos<'a>, Pos<'a>),
    lexer:  &'p mut L,
}

// Helpers

macro_rules! expect {
    ( $parser:ident>, $desc:expr, $( $( $token:pat )|+ ),+ ) => {
        match $parser.token {
            $(
                $( $token )|+ => $parser.advance(),
            ),+
            _ => expected!($parser, $desc)
        }
    };
    ( $parser:ident@, $desc:expr, $( $( $token:pat )|+ ),+ ) => {
        match $parser.token {
            $(
                $( $token )|+ => {},
            ),+
            _ => expected!($parser, $desc)
        }
    };
    ( $parser:ident>, $desc:expr, $( $( $token:pat )|+ => $e:expr ),+ ) => {
        match $parser.token {
            $(
                $( $token )|+ => { $parser.advance(); $e },
            ),+
            _ => expected!($parser, $desc)
        }
    };
    ( $parser:ident@, $desc:expr, $( $( $token:pat )|+ => $e:expr ),+ ) => {
        match $parser.token {
            $(
                $( $token )|+ => $e,
            ),+
            _ => expected!($parser, $desc)
        }
    };
}

macro_rules! expected {
    ( $parser:ident, $e:expr ) => {{
        $parser.expected($e);
        return Err(())
    }};
}

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

    // stmts:
    //   EOS? ( stmt EOS )* stmt? end
    //
    //   Note: Lexer will not return consecutive EOS.
    //
    fn parse_stmts_until(&mut self,
                         end: Token<'a>,
                         description: &str
                        ) -> Many<Stmt<'a>> {
        let pos = self.span.0;
        let mut stmts = vec![];

        // EOS?
        if self.token == Token::Eos { self.advance() }

        // ( stmt EOS )* stmt? end
        loop {
            // end?
            if self.token == end { return Ok(stmts); }

            // stmt
            stmts.push(try!(self.parse_stmt()));

            // end?
            if self.token == end { return Ok(stmts); }

            // EOS => loop
            match self.token {
                Token::Eos => self.advance(),
                _          => expected!(self, description)
            }
        }
    }

    // stmt:
    //   typedef
    //   expr     (later)
    //
    fn parse_stmt(&mut self) -> One<Stmt<'a>> {
        let pos = self.span.0;
        match self.token {
            Token::KwType => self.parse_typedef(),
            _ => {
                match self.parse_expr() {
                    Ok(expr) => Ok(Box::new(Stmt::Eval(pos, expr))),
                    _        => expected!(self, "statement or expression")
                }
            }
        }
    }

    // typedef:
    //   'type' id '=' id  // TODO: needs to be type, not id
    //
    fn parse_typedef(&mut self) -> One<Stmt<'a>> {
        let pos = self.span.0;

        expect!(self>, "'type' keyword", Token::KwType);
        let name =
        expect!(self>, "identifier", Token::Id(n) => n);

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

    // expr:
    //   expr INFIX expr
    //   expr POSTFIX
    //   primary-expr
    //
    #[inline(always)]
    fn parse_expr(&mut self) -> One<Expr<'a>> {
        self.parse_expr_prec(0)
    }

    // Precedence Climbing Algorithm
    //   + support for postfix operators
    //
    // http://www.engr.mun.ca/~theo/Misc/exp_parsing.htm
    // http://eli.thegreenplace.net/2012/08/02/parsing-expressions-by-precedence-climbing
    //
    fn parse_expr_prec(&mut self, min_prec: u8) -> One<Expr<'a>> {

        // Parse initial left operand
        let mut expr = try!(self.parse_expr_primary());

        // Parse operators and right operands
        loop {
            // Is token a binary operator?
            let (prec, assoc, fix) = match Self::op_info(&self.token) {
                Some(i) => i,
                None    => break
            };

            // Is operator's precedence at least minimum?
            if prec < min_prec {
                break
            }

            // Consume operator
            self.advance();

            // Construct expression
            match fix {
                Infix(ctor) => {
                    // Parse right operand
                    let rhs = try!(self.parse_expr_prec(match assoc {
                        Left  => prec + 1,
                        Right => prec
                    }));

                    // Append to expression
                    expr = Box::new(ctor(expr, rhs, None));
                },
                Postfix(ctor) => {
                    // Append to expression
                    expr = Box::new(ctor(expr, None))
                },
            }
        }

        Ok(expr)
    }

    // primary-expr:
    //   PREFIX expr
    //   INT
    //   '(' expr ')'  (future)
    //
    fn parse_expr_primary(&mut self) -> One<Expr<'a>> {

        // Unary operator
        if let Some((prec, ctor)) = Self::prefix_op_info(&self.token) {
            self.advance();
            let expr = try!(self.parse_expr_prec(prec));
            return Ok(Box::new(ctor(expr, None)));
        }

        // Atom
        match self.token {
            // ID
            Token::Id(x) => {
                self.advance();
                Ok(Box::new(Expr::Ident(x)))
            },
            // INT
            Token::Int(x) => {
                self.advance();
                Ok(Box::new(Expr::Int(BigInt::from(x))))
            },
            // '(' expr ')'
            Token::ParenL => {
                self.advance();
                let expr = try!(self.parse_expr());
                expect!(self>, "')'", Token::ParenR);
                Ok(expr)
            },
            _ => expected!(self, "expression")
        }
    }

    fn op_info(token: &Token<'a>) -> Option<(u8, Assoc, Fix<'a>)> {
        Some(match *token {             //ARITY PREC ASSOC
            //Token::Dot        => (12, Left,  Infix   (Expr::Add)       ),

            //Token::At         => (11, Right, Infix   (Expr::Add)       ),

            Token::PlusPlus     => (10, Left,  Postfix (Expr::Increment) ),
            Token::MinusMinus   => (10, Left,  Postfix (Expr::Decrement) ),

            // (prefix operators not shown)

            Token::Star         => ( 8, Left,  Infix   (Expr::Multiply)  ),
            Token::Slash        => ( 8, Left,  Infix   (Expr::Divide)    ),
            Token::Percent      => ( 8, Left,  Infix   (Expr::Modulo)    ),

            Token::Plus         => ( 7, Left,  Infix   (Expr::Add)       ),
            Token::Minus        => ( 7, Left,  Infix   (Expr::Subtract)  ),

            Token::LessLess     => ( 6, Left,  Infix   (Expr::ShiftL)    ),
            Token::MoreMore     => ( 6, Left,  Infix   (Expr::ShiftR)    ),

            Token::Ampersand    => ( 5, Left,  Infix   (Expr::BitAnd)    ),

            Token::Caret        => ( 4, Left,  Infix   (Expr::BitXor)    ),

            Token::Pipe         => ( 3, Left,  Infix   (Expr::BitOr)     ),

            Token::DotTilde     => ( 2, Left,  Infix   (Expr::BitChange) ),
            Token::DotBang      => ( 2, Left,  Infix   (Expr::BitClear)  ),
            Token::DotEqual     => ( 2, Left,  Infix   (Expr::BitSet)    ),
            Token::DotQuestion  => ( 2, Left,  Infix   (Expr::BitTest)   ),

            Token::Question     => ( 1, Left,  Postfix (Expr::Test)      ),
            Token::LessMore     => ( 1, Left,  Infix   (Expr::Compare)   ),

            Token::Equal        => ( 0, Right, Infix   (Expr::Move)      ),
            _                   => return None
        })
    }

    fn prefix_op_info(token: &Token<'a>) -> Option<(u8, Unary<'a>)> {
        Some(match *token {
            Token::Bang         => (9, Expr::Clear     ),
            Token::Tilde        => (9, Expr::Complement),
            Token::Plus         => (9, pass_through    ),
            Token::Minus        => (9, Expr::Negate    ),
            //Token::Ampersand  => (9, Expr::???       ),
            _                   => return None
        })
    }
}

#[derive(Clone, Copy, Debug)]
enum Assoc { Left, Right }

#[derive(Clone, Copy, Debug)]
enum Fix<'a> {
      Infix (Binary<'a>),
    Postfix ( Unary<'a>),
}

type Binary<'a> = fn(Box<Expr<'a>>, Box<Expr<'a>>, Option<&'a str>) -> Expr<'a>;
type  Unary<'a> = fn(Box<Expr<'a>>,                Option<&'a str>) -> Expr<'a>;

fn pass_through<'a>(expr: Box<Expr<'a>>, _: Option<&'a str>) -> Expr<'a> {
    *expr
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
