#![allow(dead_code)]
#![allow(unused_variables)]

use std::str::CharIndices;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Token {
    IntLit (i32),
    Plus,
    Minus,
    Eof
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct SrcId(pub u16);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Span(pub u32, pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SrcLoc {
    pub src_id: SrcId,
    pub span:   Span
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TokenSite {
    pub token:  Token,
    pub loc:    SrcLoc
}

pub struct Lex<'a> {
    next_char:  Option<char>,
    chars:      CharIndices<'a>,
    loc:        SrcLoc
}

trait StrExt {
    fn lex(&self, src_id: SrcId) -> Lex;
}

impl StrExt for str {
    fn lex(&self, src_id: SrcId) -> Lex {
        let mut l = Lex {
            next_char:  None,
            chars:      self.char_indices(),
            loc:        SrcLoc { src_id: src_id, span: Span(0, 0) }
        };
        l.consume();
        l
    }
}

impl<'a> Iterator for Lex<'a> {
    type Item = TokenSite;

    fn next(&mut self) -> Option<TokenSite> {
        Some(self.advance())
    }
}

impl<'a> Lex<'a> {
    fn advance(&mut self) -> TokenSite {
        match self.next_char {
            Some(c) => match c {
                '+' => { self.consume().produce(Token::Plus) },
                '-' => { self.consume().produce(Token::Minus) },
                 _  => { self.consume().produce(Token::Eof) }
            },
            None => { self.produce(Token::Eof) }
        }
    }

    fn accept(&mut self, c: char) -> bool {
        let ok = self.next_char == Some(c);
        if  ok { self.consume(); }
        ok
    }

    fn consume(&mut self) -> &mut Self {
        match self.chars.next() {
           Some((i, c)) => {
               self.next_char  = Some(c);
               self.loc.span.1 = i as u32;
           },
           None => {
               self.next_char = None;
           }
        }
        self
    }

    fn produce(&self, t: Token) -> TokenSite {
        TokenSite { token: t, loc: self.loc }
    }
}

#[test]
fn test_something() {
    let mut l = "-+".lex(SrcId(42));
    l.next();
    let s = l.next();

    println!("");
    println!("{:?}", s);
    println!("");

    assert!(match s { Some(t) => t.token == Token::Plus, _ => false });
}


