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
    Lex_Invalid,
    Lex_NumInvalid,
    Lex_NumOverflow,
    Lex_RawUnterminated,
    Lex_StrUnterminated,
    Lex_CharUnterminated,
    Lex_CharLength,
    Lex_EscInvalid,
    Lex_EscUnterminated,
    Lex_EscOverflow,
    // Parser Messages
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

    pub fn err_invalid_char(&mut self, p: Pos, c: char) {
        self.add(p, Error, Lex_Invalid, format!(
            "Unrecognized character: '{}'", c
        ));
    }

    pub fn err_num_invalid_char(&mut self, p: Pos, c: char) {
        self.add(p, Error, Lex_NumInvalid, format!(
            "Unrecognized character in number literal: '{}'", c
        ));
    }

    pub fn err_num_overflow(&mut self, p: Pos) {
        self.add(p, Error, Lex_NumOverflow,
            "Overflow in number literal.  Integers are unsigned 64-bit."
        );
    }

//    Lex_RawUnterminated  => "Unterminated raw block.",
//    Lex_StrUnterminated  => "Unterminated string literal.",
//    Lex_CharUnterminated => "Unterminated character literal.",
//    Lex_CharLength       => "Invalid character literal length.  \
//                             Character literals must contain exactly one character.",
//    Lex_EscInvalid       => "Overflow in Unicode escape sequence.  \
//                             The maximum permitted is \\u{10FFFF}.",
//    Lex_EscUnterminated  => "Incomplete escape sequence.",
//    Lex_EscOverflow      => "Invalid escape sequence."
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

        m.err_invalid_char(p, 'c');

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

        m.err_invalid_char(p, 'c');
        p.advance('c');
        m.err_invalid_char(p, 'd');

        assert_eq!(2,    m.error_count());
        assert_eq!(true, m.has_errors());
        assert_eq!(
            "(file):1:1: E000: Unrecognized character: 'c'\n\
             (file):1:2: E000: Unrecognized character: 'd'\n",
            m.to_string()
        );
    }
}

