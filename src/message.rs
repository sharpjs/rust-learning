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

// Defines a messages enum and a map from enum items to strings.
macro_rules! messages {
    ( $( $id:ident => $str:expr ),* ) => (
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        #[repr(u8)]
        pub enum Message { $( $id ),* }

        static_array! { MESSAGES: [&'static str] = $( $str ),* }
    );
}

// Defines a static array with automatic count.
macro_rules! static_array {
    [ $id:ident: [$t:ty] = $( $e:expr ),* ] => (
        static $id: [$t; count!($( $e ),*)] = [$( $e ),*];
    );
}

// Counts arguments.  Yields an expression like: 1 + 1 + 1
macro_rules! count {
    ( $e:expr, $( $x:tt )+ ) => (1 + count!($( $x )+));
    ( $e:expr              ) => (1);
    (                      ) => (0);
}

messages! {
    Lex_Invalid          => "Unrecognized character.",
    Lex_NumInvalid       => "Invalid character in numeric literal.",
    Lex_NumOverflow      => "Overflow in numeric literal.  \
                             Aex integers are unsigned 64-bit.",
    Lex_CharUnterminated => "Unterminated character literal.",
    Lex_CharLength       => "Invalid character literal length.  \
                             Character literals must contain exactly one character.",
    Lex_StrUnterminated  => "Overflow in Unicode escape sequence.  \
                             The maximum permitted is \\u{10FFFF}.",
    Lex_EscInvalid       => "Unterminated string literal.",
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

