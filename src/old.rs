#![allow(dead_code)]
#![allow(unused_variables)]

// ----------------------------------------------------------------------------
// Character Classifier

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Char {
    Other,  // other character, valid only in strings
    Eof,    // end of file
    Space,  // whitespace: general
    LF,     // whitespace: line feed
    CR,     // whitespace: carriage return
    Zero,   // digit: zero
    Digit,  // digit: non-zero
    Plus,   // +
    Minus,  // -
}
use Char::*;

static CHARS: [Char; 128] = [
    Other, Other, Other, Other, Other, Other, Other, Other, // ........
    Other, Space, LF   , Other, Other, CR   , Other, Other, // .tn..r..
    Other, Other, Other, Other, Other, Other, Other, Other, // ........
    Other, Other, Other, Other, Other, Other, Other, Other, // ........
    Space, Other, Other, Other, Other, Other, Other, Other, //  !"#$%&'
    Other, Other, Other, Plus , Other, Minus, Other, Other, // ()*+,-./
    Zero , Digit, Digit, Digit, Digit, Digit, Digit, Digit, // 01234567
    Digit, Digit, Other, Other, Other, Other, Other, Other, // 89:;<=>?
    Other, Other, Other, Other, Other, Other, Other, Other, // @ABCDEFG
    Other, Other, Other, Other, Other, Other, Other, Other, // HIJKLMNO
    Other, Other, Other, Other, Other, Other, Other, Other, // PQRSTUVW
    Other, Other, Other, Other, Other, Other, Other, Other, // XYZ[\]^_
    Other, Other, Other, Other, Other, Other, Other, Other, // `abcdefg
    Other, Other, Other, Other, Other, Other, Other, Other, // hijklmno
    Other, Other, Other, Other, Other, Other, Other, Other, // pqrstuvw
    Other, Other, Other, Other, Other, Other, Other, Other, // xyz{|}~. <- DEL
];

impl Char {
    fn from_char(c: char) -> Char {
        let i = c as usize;
        if (i & 0x7F) == i {
            CHARS[i]    // 7-bit ASCII
        } else {
            Other       // extended characers
        }
    }
}

#[test]
fn test_from_char_space() {
    assert_eq!(Char::from_char(' '), Space);
}

#[test]
fn test_from_char_nl() {
    assert_eq!(Char::from_char('\r'), CR);
}

#[test]
fn test_from_char_zero() {
    assert_eq!(Char::from_char('0'), Zero);
}

#[test]
fn test_from_char_digit() {
    assert_eq!(Char::from_char('9'), Digit);
}

// ----------------------------------------------------------------------------
// Lexer

use std::str::CharIndices;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Token {
    IntLit (i32),   // integer literal
    Plus,           // +
    Minus,          // -
    Eof,            // end-of-file marker
    Error           // error
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    Initial
}
use State::*;

type Action = Option<fn() -> Option<Token>>;

static TABLE: [(State, Action); 8] = [
    // Next State   Action          // Initial
    (Initial,       Some(diddly)),  // Other
    (Initial,       None),          // Space
    (Initial,       None),          // LF
    (Initial,       None),          // CR
    (Initial,       None),          // Zero
    (Initial,       None),          // Digit
    (Initial,       None),          // Plus
    (Initial,       None),          // Minus
];

fn diddly() -> Option<Token> { None }

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
    fn blop(&mut self) -> Token {
        loop {
            let c = match self.next_char {
                Some(ch) => Char::from_char(ch),
                None     => Eof
            };

            let (s, a) = TABLE[c as usize];

            if let Some(f) = a {
                if let Some(t) = f() {
                    return t;
                }
            }

            self.consume();
        }
    }

    fn advance(&mut self) -> TokenSite {
        match self.next_char {
            Some(c) => match c {
                '0' => { self.consume().produce(Token::IntLit(0)) }
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

#[test]
fn test_something_2() {
    let mut l = "+0".lex(SrcId(42));
    l.next();
    let s = l.next();

    println!("");
    println!("{:?}", s);
    println!("");

    assert!(match s { Some(t) => t.token == Token::IntLit(0), _ => false });
}

