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

use std::borrow::{Cow};
use std::fmt::{self, Display};
use std::io::{stderr, Write};

use aex::pos::Pos;

use self::MessageId::*;
use self::MessageLevel::*;

#[derive(Clone)]
pub struct Messages<'a> {
    messages:    Vec<Message<'a>>,
    error_count: usize
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Message<'a> {
    pub position:   Pos<'a>,
    pub level:      MessageLevel,
    pub id:         MessageId,
    pub text:       Cow<'static, str>,
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(u8)]
pub enum MessageLevel { Warning, Error }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum MessageId {
    // Lexer Messages
    Unrec,
    UnrecNum,
    UnrecEsc,
    UntermChar,
    UntermStr,
    UntermRaw,
    UntermEsc,
    CharLength,
    OverflowNum,
    OverflowEsc,

    // Parser Messages
    //   (none yet)

    // Declaration Analysis Messages
    TypeRedefined,
    SymRedefined,

    // Semantic Analysis Messages
    TypeMismatch,
}

impl<'a> Messages<'a> {
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

    fn add<T>(&mut self, p: Pos<'a>, l: MessageLevel, i: MessageId, t: T)
             where T: Into<Cow<'static, str>> {
        self.messages.push(Message {
            position: p, level: l, id: i, text: t.into()
        });
        if l >= Error {
            self.error_count += 1;
        }
    }

    pub fn err_unrec(&mut self, p: Pos<'a>, c: char) {
        self.add(p, Error, Unrec, format!(
            "Unrecognized character: '{}'", c
        ));
    }

    pub fn err_unrec_num(&mut self, p: Pos<'a>, c: char) {
        self.add(p, Error, UnrecNum, format!(
            "Unrecognized character in number literal: '{}'", c
        ));
    }

    pub fn err_unrec_esc(&mut self, p: Pos<'a>, c: char) {
        self.add(p, Error, UnrecEsc, format!(
            "Unrecognized character in escape sequence: '{}'", c
        ));
    }

    pub fn err_unterm_char(&mut self, p: Pos<'a>) {
        self.add(p, Error, UntermChar,
            "Unterminated character literal."
        );
    }

    pub fn err_unterm_str(&mut self, p: Pos<'a>) {
        self.add(p, Error, UntermStr,
            "Unterminated string literal."
        );
    }

    pub fn err_unterm_raw(&mut self, p: Pos<'a>) {
        self.add(p, Error, UntermRaw,
            "Unterminated raw block."
        );
    }

    pub fn err_unterm_esc(&mut self, p: Pos<'a>) {
        self.add(p, Error, UntermEsc,
            "Unterminated escape sequence."
        );
    }

    pub fn err_length_char(&mut self, p: Pos<'a>) {
        self.add(p, Error, CharLength,
            "Invalid character literal length. \
             Character literals must contain exactly one character."
        );
    }

    pub fn err_overflow_num(&mut self, p: Pos<'a>) {
        self.add(p, Error, OverflowNum,
            "Overflow in number literal.  Integers are unsigned 64-bit."
        );
    }

    pub fn err_overflow_esc(&mut self, p: Pos<'a>) {
        self.add(p, Error, OverflowEsc,
            "Overflow in Unicode escape sequence. \
             The maximum permitted is \\u{10FFFF}."
        );
    }

    pub fn err_type_redefined(&mut self, p: &Pos<'a>, name: &str) {
        self.add(p.clone(), Error, TypeRedefined, format!(
            // TODO: Don't need clone
            "Type already defined: {}", name
        ));
    }

    pub fn err_sym_redefined(&mut self, p: &Pos<'a>, name: &str) {
        self.add(p.clone(), Error, SymRedefined, format!(
            // TODO: Don't need clone
            "Symbol already defined: {}", name
        ));
    }

    pub fn err_type_mismatch(&mut self, p: Pos<'a>) {
        self.add(p, Error, TypeMismatch,
            "Type mismatch."
        );
    }

    pub fn print(&self) {
        let r = write!(stderr(), "{}", self);
        if let Err(_) = r { /*ignore*/ }
    }
}

impl<'a> Display for Messages<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for m in &self.messages {
            try!(writeln!(f, "{}", m));
        }
        Ok(())
    }
}

impl<'a> Display for Message<'a> {
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
    use aex::pos::Pos;

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
        let     p = Pos::bof("f");

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
        let mut p = Pos::bof("f");

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

