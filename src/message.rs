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

static_array! { X: [i32] = 1, 2, 3 }

messages! {
    Lex_Invalid         => "",
    Lex_NumInvalid      => "",
    Lex_NumOverflow     => "",
    Lex_CharUntermnated => "",
    Lex_CharLength      => "",
    Lex_StrUnterminated => "",
    Lex_EscInvalid      => "",
    Lex_EscUnterminated => "",
    Lex_EscOverflow     => ""
}

