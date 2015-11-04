// Utilities
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

// -----------------------------------------------------------------------------
// Pos - a position within a character stream

use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub byte:   usize,  // 0-based byte offset
    pub line:   u32,    // 1-based line number
    pub column: u32,    // 1-based column number
}

impl Pos {
    #[inline]
    pub fn bof() -> Pos {
        Pos { byte: 0, line: 1, column: 1 }
    }

    #[inline]
    pub fn advance(&mut self, c: char) {
        self.byte   += c.len_utf8();
        self.column += 1;
    }

    #[inline]
    pub fn newline(&mut self) {
        self.line  += 1;
        self.column = 1;
    }
}

impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}|{}:{}>", self.byte, self.line, self.column)
    }
}

#[cfg(test)]
mod tests {

    #[cfg(test)]
    mod pos {
        use super::super::*;

        #[test]
        fn bof() {
            let p = Pos::bof();
            assert_eq!(p, Pos { byte: 0, line: 1, column: 1 });
        }

        #[test]
        fn advance() {
            let mut p = Pos::bof();
            p.advance('a');
            assert_eq!(p, Pos { byte: 1, line: 1, column: 2 });
            p.advance('\u{10ABCD}');
            assert_eq!(p, Pos { byte: 5, line: 1, column: 3 });
        }

        #[test]
        fn newline() {
            let mut p = Pos::bof();
            p.advance('\n');
            p.newline();
            assert_eq!(p, Pos { byte: 1, line: 2, column: 1 });
        }
    }
}

