// Lexical Tokens
//
// This file is part of AEx.
// Copyright (C) 2017 Jeffrey Sharp
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

use std::cell::RefCell;
use std::collections::HashMap;

use num::{self, BigInt, ToPrimitive};

use aex::operator::{OperatorEntry, OperatorTable};
use aex::mem::StringInterner;
use aex::message::Messages;
use aex::source::*;

// Placeholder
pub struct Compiler<'a> {
    strings:    StringInterner,
    operators:  OperatorTable,
    log:        RefCell<Messages<'a>>
}

impl<'a> Compiler<'a> {
    pub fn new() -> Self {
        Compiler {
            strings: StringInterner::new(),
            operators: OperatorTable::new(),
            log: RefCell::new(Messages::new()),
        }
    }
}

// -----------------------------------------------------------------------------
// Token

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Token<'a> {
    Id   (&'a str),         // Identifier
    Flag (&'a str),         // Condition flag

    Int  (BigInt),          // Literal: integer
    Char (char),            // Literal: character
    Str  (&'a str),         // Literal: string

    KwType,                 // Keyword: type
    KwStruct,               // Keyword: struct
    KwUnion,                // Keyword: union
    KwIf,                   // Keyword: if
    KwElse,                 // Keyword: else
    KwLoop,                 // Keyword: loop
    KwWhile,                // Keyword: while
    KwBreak,                // Keyword: break
    KwContinue,             // Keyword: continue
    KwReturn,               // Keyword: return
    KwJump,                 // Keyword: jump

    BraceL,                 // {
    BraceR,                 // }
    ParenL,                 // (
    ParenR,                 // )
    BracketL,               // [
    BracketR,               // ]
  //Dot,                    // .
  //At,                     // @
  //Equal,                  // =
  //EqualArrow,             // =>
  //MinusArrow,             // ->
    Colon,                  // :
    Comma,                  // ,

    Op(&'a OperatorEntry),  // any of: .@!~*/%+-&^|<=>?

    Eos,                    // End of statement
    Eof,                    // End of file

    Error                   // Lexical error
}

// -----------------------------------------------------------------------------
// TokenBuilder

pub struct TokenBuilder<'a> {
    // Source
    file:       &'a File<'a>,                   // source file
    start:      Pos,                            // position of token start
    current:    Pos,                            // position of current character

    // Accumulators
    number:     BigInt,                         // number builder
    buffer:     String,                         // string builder

    // Language
    keywords:   HashMap<&'a str, Token<'a>>,    // keyword table
    operators:  &'a OperatorTable,              // operator table

    // Session
    strings:    &'a StringInterner,             // string interner
    log:        &'a RefCell<Messages<'a>>       // message log
}

impl<'a> TokenBuilder<'a> {
    pub fn new(compiler: &'a Compiler <'a>,
           file:     &'a File     <'a>,
          ) -> Self {
        TokenBuilder {
            file:      file,
            start:     Pos::bof(),
            current:   Pos::bof(),
            buffer:    String::with_capacity(128),
            number:    num::zero(),
            keywords:  keywords(&compiler.strings),
            operators: &compiler.operators,
            strings:   &compiler.strings,
            log:       &compiler.log,
        }
    }

    // Position actions

    #[inline]
    pub fn start(&mut self) {
        self.start = self.current;
    }

    #[inline]
    pub fn advance(&mut self, c: char) {
        self.current.advance(c);
    }

    #[inline]
    pub fn newline(&mut self) {
        self.current.newline();
    }

    #[inline]
    pub fn source(&self) -> Source<'a> {
        let len = self.current.byte - self.start.byte;
        Source::File { file: self.file, pos: self.start, len: len }
    }

    // Number actions

    #[inline]
    pub fn add_dec(&mut self, c: char) {
        self.add_num(10, int_from_dg(c))
    }

    #[inline]
    pub fn add_hex_dg(&mut self, c: char) {
        self.add_num(16, int_from_dg(c))
    }

    #[inline]
    pub fn add_hex_uc(&mut self, c: char) {
        self.add_num(16, int_from_hex_uc(c))
    }

    #[inline]
    pub fn add_hex_lc(&mut self, c: char) {
        self.add_num(16, int_from_hex_lc(c))
    }

    #[inline]
    pub fn add_oct(&mut self, c: char) {
        self.add_num(8, int_from_dg(c))
    }

    #[inline]
    pub fn add_bin(&mut self, c: char) {
        self.add_num(2, int_from_dg(c))
    }

    #[inline]
    fn add_num(&mut self, base: u8, digit: u8) {
        self.number = &self.number
            * BigInt::from(base)
            + BigInt::from(digit);
    }

    #[inline]
    pub fn get_num(&mut self) -> Token<'a> {
        let number = self.number.clone();
        self.number = num::zero();
        Token::Int(number)
    }

    // Character/String Actions

    #[inline]
    pub fn add_char(&mut self, c: char) {
        self.buffer.push(c);
    }

    #[inline]
    pub fn add_esc(&mut self) -> Option<Token<'a>> {
        match self.number.to_u32() {
            Some(n) if n <= UNICODE_MAX => {
                let c = unsafe { ::std::mem::transmute(n) };
                self.buffer.push(c);
                self.number = num::zero();
                None
            },
            _ => {
                Some(self.err_overflow_esc())
            }
        }
    }

    #[inline]
    fn intern_str(&mut self) -> &'a str {
        let s = self.strings.intern(self.buffer.clone());
        self.buffer.clear();
        s
    }

    #[inline]
    pub fn get_char(&mut self) -> Token<'a> {
        let c = self.buffer.chars().next().unwrap();
        self.buffer.clear();
        Token::Char(c)
    }

    #[inline]
    pub fn get_str(&mut self) -> Token<'a> {
        Token::Str(self.intern_str())
    }

    #[inline]
    pub fn get_id_or_keyword(&mut self) -> Token<'a> {
        let s = self.intern_str();

        match self.keywords.get(&s) {
            Some(k) => k.clone(),
            None    => Token::Id(s)
        }
    }

    // Error Actions

    pub fn err_unrec(&mut self, c: char) -> Token<'a> {
        self.log.borrow_mut().err_unrec(self.source(), c);
        Token::Error
    }

    pub fn err_unrec_num(&mut self, c: char) -> Token<'a> {
        self.log.borrow_mut().err_unrec_num(self.source(), c);
        Token::Error
    }

    pub fn err_unrec_esc(&mut self, c: char) -> Token<'a> {
        self.log.borrow_mut().err_unrec_esc(self.source(), c);
        Token::Error
    }

    pub fn err_unterm_char(&mut self) -> Token<'a> {
        self.log.borrow_mut().err_unterm_char(self.source());
        Token::Error
    }

    pub fn err_unterm_str(&mut self) -> Token<'a> {
        self.log.borrow_mut().err_unterm_str(self.source());
        Token::Error
    }

    pub fn err_unterm_esc(&mut self) -> Token<'a> {
        self.log.borrow_mut().err_unterm_esc(self.source());
        Token::Error
    }

    pub fn err_length_char(&mut self) -> Token<'a> {
        self.log.borrow_mut().err_length_char(self.source());
        Token::Error
    }

    pub fn err_overflow_esc(&mut self) -> Token<'a> {
        self.log.borrow_mut().err_overflow_esc(self.source());
        Token::Error
    }
}

const UNICODE_MAX: u32 = 0x10FFFF;

#[inline]
fn int_from_dg(c: char) -> u8 {
    c as u8 - 0x30 // c - '0'
}

#[inline]
fn int_from_hex_uc(c: char) -> u8 {
    c as u8 - 0x37 // 10 + c - 'A'
}

#[inline]
fn int_from_hex_lc(c: char) -> u8 {
    c as u8 - 0x57 // 10 + c - 'a'
}

// -----------------------------------------------------------------------------
// Keywords

// TODO: Consider moving this to Compiler, so that targets can add custom keywords.

#[inline]
fn keywords<'a>(strings: &'a StringInterner) -> HashMap<&'a str, Token<'a>> {
    let mut map = HashMap::new();

    for &(key, ref tok) in KEYWORDS {
        map.insert(strings.intern_ref(key), tok.clone());
    }

    map
}

const KEYWORDS: &'static [(&'static str, Token<'static>)] = &[
    ( "type"     , Token::KwType     ),
    ( "struct"   , Token::KwStruct   ),
    ( "union"    , Token::KwUnion    ),
    ( "if"       , Token::KwIf       ),
    ( "else"     , Token::KwElse     ),
    ( "loop"     , Token::KwLoop     ),
    ( "while"    , Token::KwWhile    ),
    ( "break"    , Token::KwBreak    ),
    ( "continue" , Token::KwContinue ),
    ( "return"   , Token::KwReturn   ),
    ( "jump"     , Token::KwJump     ),
];

