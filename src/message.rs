// Messages
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

use util::Pos;
use std::borrow::{Cow};
use std::fmt::{self, Display};
use std::io::{stderr, Write};

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(u8)]
pub enum MessageLevel { Warning, Error }
use self::MessageLevel::*;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum MessageId {
    // Lexer Messages
    Lex_Unrec,
    Lex_UnrecNum,
    Lex_UnrecEsc,
    Lex_UntermChar,
    Lex_UntermStr,
    Lex_UntermRaw,
    Lex_UntermEsc,
    Lex_CharLength,
    Lex_OverflowNum,
    Lex_OverflowEsc,
    // Parser Messages
    // ...
    // Semantic Analysis Messages
    Sem_TypeMismatch,
}
use self::MessageId::*;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Message {
    pub position:   Pos,
    pub level:      MessageLevel,
    pub id:         MessageId,
    pub text:       Cow<'static, str>,
}

pub struct Messages {
    messages:    Vec<Message>,
    error_count: usize
}

impl Messages {
    pub fn new() -> Self {
        Messages {
            messages:    vec![],
            error_count: 0
        }
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    pub fn error_count(&self) -> usize {
        self.error_count
    }

    pub fn print(&self) {
        let r = write!(stderr(), "{}", self);
        if let Err(_) = r { /*ignore*/ }
    }

    fn add<T>(&mut self, p: Pos, l: MessageLevel, i: MessageId, t: T)
    where T: Into<Cow<'static, str>> {
        self.messages.push(Message {
            position: p, level: l, id: i, text: t.into()
        });
        if l >= Error {
            self.error_count += 1;
        }
    }

    pub fn err_unrec(&mut self, p: Pos, c: char) {
        self.add(p, Error, Lex_Unrec, format!(
            "Unrecognized character: '{}'", c
        ));
    }

    pub fn err_unrec_num(&mut self, p: Pos, c: char) {
        self.add(p, Error, Lex_UnrecNum, format!(
            "Unrecognized character in number literal: '{}'", c
        ));
    }

    pub fn err_unrec_esc(&mut self, p: Pos, c: char) {
        self.add(p, Error, Lex_UnrecEsc, format!(
            "Unrecognized character in escape sequence: '{}'", c
        ));
    }

    pub fn err_unterm_char(&mut self, p: Pos) {
        self.add(p, Error, Lex_UntermChar,
            "Unterminated character literal."
        );
    }

    pub fn err_unterm_str(&mut self, p: Pos) {
        self.add(p, Error, Lex_UntermStr,
            "Unterminated string literal."
        );
    }

    pub fn err_unterm_raw(&mut self, p: Pos) {
        self.add(p, Error, Lex_UntermRaw,
            "Unterminated raw block."
        );
    }

    pub fn err_unterm_esc(&mut self, p: Pos) {
        self.add(p, Error, Lex_UntermEsc,
            "Unterminated escape sequence."
        );
    }

    pub fn err_length_char(&mut self, p: Pos) {
        self.add(p, Error, Lex_CharLength,
            "Invalid character literal length. \
             Character literals must contain exactly one character."
        );
    }

    pub fn err_overflow_num(&mut self, p: Pos) {
        self.add(p, Error, Lex_OverflowNum,
            "Overflow in number literal.  Integers are unsigned 64-bit."
        );
    }

    pub fn err_overflow_esc(&mut self, p: Pos) {
        self.add(p, Error, Lex_OverflowEsc,
            "Overflow in Unicode escape sequence. \
             The maximum permitted is \\u{10FFFF}."
        );
    }

    pub fn err_type_mismatch(&mut self, p: Pos) {
        self.add(p, Error, Sem_TypeMismatch,
            "Type mismatch."
        );
    }
}

impl Display for Messages {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for m in &self.messages {
            try!(writeln!(f, "{}", m));
        }
        Ok(())
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}: {}{:03}: {}",
            "(file)",
            self.position.line,
            self.position.column,
            match self.level { Warning => 'W', Error => 'E' },
            self.id as u16,
            self.text
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use util::Pos;

    #[test]
    fn messages_empty() {
        let m = Messages::new();

        assert_eq!(0,     m.error_count());
        assert_eq!(false, m.has_errors());
        assert_eq!("",    m.to_string());
    }

    #[test]
    fn messages_single() {
        let mut m = Messages::new();
        let     p = Pos::bof();

        m.err_unrec(p, 'c');

        assert_eq!(1,     m.error_count());
        assert_eq!(true,  m.has_errors());
        assert_eq!(
            "(file):1:1: E000: Unrecognized character: 'c'\n",
            m.to_string()
        );
    }

    #[test]
    fn messages_multiple() {
        let mut m = Messages::new();
        let mut p = Pos::bof();

        m.err_unrec(p, 'c');
        p.advance('c');
        m.err_unrec(p, 'd');

        assert_eq!(2,    m.error_count());
        assert_eq!(true, m.has_errors());
        assert_eq!(
            "(file):1:1: E000: Unrecognized character: 'c'\n\
             (file):1:2: E000: Unrecognized character: 'd'\n",
            m.to_string()
        );
    }
}

