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

use std::fmt::{self, Debug, Display, Formatter};
use std::fs;
use std::io::{self, Read};

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum Source<'a> {
    // Intrinsic language feature
    BuiltIn,

    // In source file
    File {
        file: &'a File<'a>, // source file
        pos:  Pos,          // position within file
        len:  usize         // length in bytes
    }
}

impl<'a> Display for Source<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Source::BuiltIn => {
                f.write_str("(built-in)")
            },
            Source::File { file, pos, len } => {
                write!(f, "{}:{}", file, pos)
            },
        }
    }
}

impl<'a> Debug for Source<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Source::BuiltIn => {
                f.write_str("Source::BuiltIn")
            },
            Source::File { file, pos, len } => {
                write!(f, "Source::File({}:{}+{})", file, pos, len)
            },
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct File<'a> {
    name: &'a str,
    data: String,
}

impl<'a> File<'a> {
    pub fn from_stdin() -> Self {
        Self::new("(stdin)", io::stdin())
    }

    pub fn from_path(path: &'a str) -> Self {
        match fs::File::open(path) {
            Ok  (f) => Self::new(path, f),
            Err (e) => fail_read(path, e)
        }
    }

    pub fn new<R: Read>(name: &'a str, mut reader: R) -> Self {
        let mut data = String::new();

        match reader.read_to_string(&mut data) {
            Ok  (_) => File { name: name, data: data },
            Err (e) => fail_read(name, e)
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.name
    }

    #[inline]
    pub fn data(&self) -> &str {
        &self.data
    }
}

impl<'a> Display for File<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.name)
    }
}

impl<'a> Debug for File<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.name)
    }
}

fn fail_read(name: &str, error: io::Error) -> ! {
    panic!("error reading '{}': {}", name, error)
}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Pos {
    pub byte:   usize,      // 0-based byte offset
    pub line:   u32,        // 1-based line number
    pub column: u32,        // 1-based column number
}

impl Pos {
    #[inline]
    pub fn bof() -> Self {
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

impl<'a> Display for Pos {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl<'a> Debug for Pos {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[{}]:{}:{}", self.byte, self.line, self.column)
    }
}

// -----------------------------------------------------------------------------

#[cfg(test)]
pub mod tests {
    use super::*;

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

    #[test]
    fn fmt_display() {
        let p = Pos { byte: 1, line: 2, column: 3 };
        let s = format!("{}", &p);
        assert_eq!(s, "2:3");
    }

    #[test]
    fn fmt_debug() {
        let p = Pos { byte: 1, line: 2, column: 3 };
        let s = format!("{:?}", &p);
        assert_eq!(s, "[1]:2:3");
    }
}

