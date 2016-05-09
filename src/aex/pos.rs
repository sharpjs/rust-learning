// Source Positions
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

use std::fmt::{self, Debug, Formatter};
use std::io::{self, Write};

use aex::fmt::Format;
use aex::mem::{Name, Strings};

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum Source {
    BuiltIn,
    File {
        pos: Pos,   // position
        len: usize  // length, in bytes
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Pos {
    pub file:   Name,       // file name
    pub byte:   usize,      // 0-based byte offset
    pub line:   u32,        // 1-based line number
    pub column: u32,        // 1-based column number
}

impl Pos {
    #[inline]
    pub fn bof(file: Name) -> Self {
        Pos { file: file, byte: 0, line: 1, column: 1 }
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

impl Format<Strings> for Pos {
    fn fmt<W: Write>(&self, s: &Strings, w: &mut W) -> io::Result<()> {
        write!(w, "{}:{}:{}", &s[self.file], self.line, self.column)
    }
}

impl Debug for Pos {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}[{}]:{}:{}", self.file, self.byte, self.line, self.column)
    }
}

#[cfg(test)]
pub mod tests {
    use aex::mem::Strings;
    use aex::mem::tests::{NAME_ZERO as FILE};
    use super::*;

    pub static BOF: Source = Source::File {
        pos: Pos { file: FILE, byte: 0, line: 1, column: 1 },
        len: 0
    };

    #[test]
    fn bof() {
        let pos = Pos::bof(FILE);
        assert_eq!(pos, Pos { file: FILE, byte: 0, line: 1, column: 1 });
    }

    #[test]
    fn advance() {
        let mut pos = Pos::bof(FILE);
        pos.advance('a');
        assert_eq!(pos, Pos { file: FILE, byte: 1, line: 1, column: 2 });
        pos.advance('\u{10ABCD}');
        assert_eq!(pos, Pos { file: FILE, byte: 5, line: 1, column: 3 });
    }

    #[test]
    fn newline() {
        let mut pos = Pos::bof(FILE);
        pos.advance('\n');
        pos.newline();
        assert_eq!(pos, Pos { file: FILE, byte: 1, line: 2, column: 1 });
    }

    #[test]
    fn fmt_format() {
        let strs = Strings::with_capacity(1);
        let file = strs.intern_ref("file.ext");
        let pos  = Pos { file: file, byte: 1, line: 2, column: 3 };
        let text = format_with!(&pos, &strs);
        assert_eq!(text, "file.ext:2:3");
    }

    #[test]
    fn fmt_debug() {
        let pos  = Pos { file: FILE, byte: 1, line: 2, column: 3 };
        let text = format!("{:?}", &pos);
        assert_eq!(text, "<0>[1]:2:3");
    }
}

