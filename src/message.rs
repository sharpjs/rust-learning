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

macro_rules! messages {
    ( $( $id:ident => $str:expr ),* ) => (

        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        #[repr(u8)]
        pub enum Message { $( $id ),* }

        static MESSAGES: [&'static str; 9] = [$( $str ),*];
        // TODO: Figure out how to do a count
    );
}

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

