// Messages
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

use std::borrow::{Cow};
use std::fmt::{self, Display};
use std::io::{stderr, Write};

use aex::pos::Source;

use self::MessageId::*;
use self::MessageLevel::*;

#[derive(Clone, Debug)]
pub struct Messages<'a> {
    messages:    Vec<Message<'a>>,
    error_count: usize
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Message<'a> {
    pub source: Source<'a>,
    pub level:  MessageLevel,
    pub id:     MessageId,
    pub text:   Cow<'static, str>,
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(u8)]
pub enum MessageLevel {
    Warning,
    Error
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
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
    Expected,

    // Declaration Analysis Messages
    TypeRedefined,
    SymRedefined,

    // Semantic Analysis Messages
    IncompatibleTypes,
    ValueOutOfRange,
    NoOpForExpression,
    NoOpForSelector,
    NoOpForAddrModes,
    NoOpForOperandTypes,
    NoOpForOperandSizes,
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

    fn add<T>(&mut self, s: Source<'a>, l: MessageLevel, i: MessageId, t: T)
             where T: Into<Cow<'static, str>> {
        self.messages.push(Message {
            source: s, level: l, id: i, text: t.into()
        });
        if l >= Error {
            self.error_count += 1;
        }
    }

    pub fn err_unrec(&mut self, s: Source<'a>, c: char) {
        self.add(s, Error, Unrec, format!(
            "Unrecognized character: '{}'", c
        ));
    }

    pub fn err_unrec_num(&mut self, s: Source<'a>, c: char) {
        self.add(s, Error, UnrecNum, format!(
            "Unrecognized character in number literal: '{}'", c
        ));
    }

    pub fn err_unrec_esc(&mut self, s: Source<'a>, c: char) {
        self.add(s, Error, UnrecEsc, format!(
            "Unrecognized character in escape sequence: '{}'", c
        ));
    }

    pub fn err_unterm_char(&mut self, s: Source<'a>) {
        self.add(s, Error, UntermChar,
            "Unterminated character literal."
        );
    }

    pub fn err_unterm_str(&mut self, s: Source<'a>) {
        self.add(s, Error, UntermStr,
            "Unterminated string literal."
        );
    }

    pub fn err_unterm_raw(&mut self, s: Source<'a>) {
        self.add(s, Error, UntermRaw,
            "Unterminated raw block."
        );
    }

    pub fn err_unterm_esc(&mut self, s: Source<'a>) {
        self.add(s, Error, UntermEsc,
            "Unterminated escape sequence."
        );
    }

    pub fn err_length_char(&mut self, s: Source<'a>) {
        self.add(s, Error, CharLength,
            "Invalid character literal length. \
             Character literals must contain exactly one character."
        );
    }

    pub fn err_overflow_num(&mut self, s: Source<'a>) {
        self.add(s, Error, OverflowNum,
            "Overflow in number literal.  Integers are unsigned 64-bit."
        );
    }

    pub fn err_overflow_esc(&mut self, s: Source<'a>) {
        self.add(s, Error, OverflowEsc,
            "Overflow in Unicode escape sequence. \
             The maximum permitted is \\u{10FFFF}."
        );
    }

    pub fn err_expected(&mut self, s: Source<'a>, description: &str) {
        self.add(s, Error, Expected, format!(
            "Expected: {}", description
        ));
    }

    pub fn err_type_redefined(&mut self, s: Source<'a>, name: &str) {
        self.add(s, Error, TypeRedefined, format!(
            "Type already defined: {}", name
        ));
    }

    pub fn err_sym_redefined(&mut self, s: Source<'a>, name: &str) {
        self.add(s, Error, SymRedefined, format!(
            "Symbol already defined: {}", name
        ));
    }

    pub fn err_incompatible_types(&mut self, s: Source<'a>) {
        self.add(s, Error, IncompatibleTypes,
            "Operands are of incompatible types."
        );
    }

    pub fn err_value_out_of_range(&mut self, s: Source<'a>) {
        self.add(s, Error, ValueOutOfRange,
            "Operand value out of range."
        );
    }

    pub fn err_no_op_for_expression(&mut self, s: Source<'a>) {
        self.add(s, Error, NoOpForExpression,
            "No target instruction for the given expression form."
        );
    }

    pub fn err_no_op_for_selector(&mut self, s: Source<'a>) {
        self.add(s, Error, NoOpForSelector,
            "No target instruction for the given selector."
        );
    }

    pub fn err_no_op_for_addr_modes(&mut self, s: Source<'a>) {
        self.add(s, Error, NoOpForAddrModes,
            "No target instruction for the given addressing mode(s)."
        );
    }

    pub fn err_no_op_for_operand_types(&mut self, s: Source<'a>) {
        self.add(s, Error, NoOpForOperandTypes,
            "No target instruction for the given operand type(s)."
        );
    }

    pub fn err_no_op_for_operand_sizes(&mut self, s: Source<'a>) {
        self.add(s, Error, NoOpForOperandSizes,
            "No target instruction for the given operand size(s)."
        );
    }

    pub fn print(&self) {
        write!(stderr(), "{}", self).is_ok();
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
        write!(f, "{}: {}{:03}: {}",
            self.source,
            match self.level { Warning => 'W', Error => 'E' },
            self.id as u16,
            self.text
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aex::pos::{Pos, Source};

    #[test]
    fn messages_empty() {
        let m = Messages::new();

        assert_eq!(0,     m.error_count());
        assert_eq!(false, m.has_errors());
        assert_eq!("",    m.to_string());
    }

    #[test]
    fn messages_single() {
        let     p = Pos::bof("file");
        let     s = Source::File { pos: &p, len: 1 };
        let mut m = Messages::new();

        m.err_unrec(s, 'c');

        assert_eq!(1,     m.error_count());
        assert_eq!(true,  m.has_errors());
        assert_eq!(
            "file:1:1: E000: Unrecognized character: 'c'\n",
            m.to_string()
        );
    }

    #[test]
    fn messages_multiple() {
        let     p0 = Pos::bof("file");
        let mut p1 = Pos::bof("file");
        let mut m  = Messages::new();

        p1.advance('c');

        let s0 = Source::File { pos: &p0, len: 1 };
        let s1 = Source::File { pos: &p1, len: 1 };

        m.err_unrec(s0, 'c');
        m.err_unrec(s1, 'd');

        assert_eq!(2,    m.error_count());
        assert_eq!(true, m.has_errors());
        assert_eq!(
            "file:1:1: E000: Unrecognized character: 'c'\n\
             file:1:2: E000: Unrecognized character: 'd'\n",
            m.to_string()
        );
    }
}

