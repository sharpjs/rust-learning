// Source Positions
//
// This file is part of AEx.
// Copyright (C) 2017 Jeffrey Sharp
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

use std::fmt;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Source<'a> {
    BuiltIn,
    File {
        pos: &'a Pos<'a>,   // position
        len: usize          // length, in bytes
    }
}

impl<'a> fmt::Display for Source<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Source::BuiltIn          => write!(f, "(built-in)"),
            Source::File { pos, .. } => write!(f, "{}", pos),
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Pos<'a> {
    pub file:   &'a str,    // file name
    pub byte:   usize,      // 0-based byte offset
    pub line:   u32,        // 1-based line number
    pub column: u32,        // 1-based column number
}

impl<'a> Pos<'a> {
    #[inline]
    pub fn bof(file: &'a str) -> Self {
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

impl<'a> fmt::Display for Pos<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

impl<'a> fmt::Debug for Pos<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}[{}]:{}:{}", self.file, self.byte, self.line, self.column)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub static BOF: Source<'static> = Source::File {
        pos: &Pos { file: "f", byte: 0, line: 1, column: 1 },
        len: 0
    };

    #[test]
    fn bof() {
        let p = Pos::bof("f");
        assert_eq!(p, Pos { file: "f", byte: 0, line: 1, column: 1 });
    }

    #[test]
    fn advance() {
        let mut p = Pos::bof("f");
        p.advance('a');
        assert_eq!(p, Pos { file: "f", byte: 1, line: 1, column: 2 });
        p.advance('\u{10ABCD}');
        assert_eq!(p, Pos { file: "f", byte: 5, line: 1, column: 3 });
    }

    #[test]
    fn newline() {
        let mut p = Pos::bof("f");
        p.advance('\n');
        p.newline();
        assert_eq!(p, Pos { file: "f", byte: 1, line: 2, column: 1 });
    }

    #[test]
    fn fmt_display() {
        let p = Pos { file: "f", byte: 1, line: 2, column: 3 };
        let s = format!("{}", &p);
        assert_eq!(s, "f:2:3");
    }

    #[test]
    fn fmt_debug() {
        let p = Pos { file: "f", byte: 1, line: 2, column: 3 };
        let s = format!("{:?}", &p);
        assert_eq!(s, "f[1]:2:3");
    }
}

