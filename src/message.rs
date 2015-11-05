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
use std::borrow::Cow;
use std::io::{stderr, Write};

pub struct Messages {
    messages:    Vec<Message_>,
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
        for m in &self.messages {
            let r = writeln!(stderr(), "{}:{}: {:?}: {}",
                m.position.line,
                m.position.column,
                m.level,
                m.text
            );

            if let Err(_) = r { return }
        }
    }
}

fn format_message(m: &Message_) -> &str { &m.text }

pub struct Message_ {
    pub id:         MessageId,
    pub level:      MessageLevel,
    pub position:   Pos,
    pub text:       Cow<'static, str>,
}

impl Message_ {
}

pub type MessageId = Message;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MessageLevel { Warning, Error }

// Defines a messages enum and a map from enum items to strings.
macro_rules! messages {
    { $( $id:ident => $str:expr ),* } => {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        #[repr(u8)]
        pub enum Message {
            $( $id ),*
        }

        const MESSAGES: &'static [&'static str] = &[
            $( $str ),*
        ];
    };
}

messages! {
    Lex_Invalid          => "Unrecognized character.",
    Lex_NumInvalid       => "Invalid character in numeric literal.",
    Lex_NumOverflow      => "Overflow in numeric literal.  \
                             Aex integers are unsigned 64-bit.",
    Lex_RawUnterminated  => "Unterminated raw block.",
    Lex_StrUnterminated  => "Unterminated string literal.",
    Lex_CharUnterminated => "Unterminated character literal.",
    Lex_CharLength       => "Invalid character literal length.  \
                             Character literals must contain exactly one character.",
    Lex_EscInvalid       => "Overflow in Unicode escape sequence.  \
                             The maximum permitted is \\u{10FFFF}.",
    Lex_EscUnterminated  => "Incomplete escape sequence.",
    Lex_EscOverflow      => "Invalid escape sequence."
}

impl Message {
    #[inline]
    fn number(self) -> u8 { self as u8 }

    #[inline]
    fn text(self) -> &'static str { MESSAGES[self as usize] }
}

impl Into<u8> for Message {
    #[inline]
    fn into(self) -> u8 { self.number() }
}

impl Into<&'static str> for Message {
    #[inline]
    fn into(self) -> &'static str { self.text() }
}

