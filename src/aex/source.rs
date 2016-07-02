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

impl<'a> AsRef<str> for Source<'a> {
    fn as_ref(&self) -> &str {
        match *self {
            Source::BuiltIn => {
                ""
            },
            Source::File { file, pos, len } => {
                &file.data()[(pos.byte)..(pos.byte + len)]
            }
        }
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
                write!(f, "{:?}:{:?}+{}", file, pos, len)
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
        write!(f, "{}({})", self.name, self.data.len())
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
        write!(f, "{}:{}[{}]", self.line, self.column, self.byte)
    }
}

// -----------------------------------------------------------------------------

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::io::Cursor;

    // -------------------------------------------------------------------------

    #[test]
    fn source_text() {
        let f = File { name: "f", data: "abc".into() };
        let s = Source::File { file: &f, pos:  Pos::bof(), len: 2 };
        let t = s.as_ref();
        assert_eq!(t, "ab")
    }

    #[test]
    fn source_fmt_display() {
        let f = File { name: "f", data: "abc".into() };
        let s = Source::File { file: &f, pos:  Pos::bof(), len: 2 };
        let s = format!("{}", s);
        assert_eq!(s, "f:1:1")
    }

    #[test]
    fn source_fmt_debug() {
        let f = File { name: "f", data: "abc".into() };
        let s = Source::File { file: &f, pos:  Pos::bof(), len: 2 };
        let s = format!("{:?}", s);
        assert_eq!(s, "f(3):1:1[0]+2")
    }

    // -------------------------------------------------------------------------

    #[test]
    fn file_new() {
        let c = Cursor::new("abc");
        let f = File::new("f", c);
        assert_eq!(f, File { name: "f", data: "abc".into() });
    }

    #[test]
    fn file_from_path() {
        let f = File::from_path("test/test.txt");
        assert_eq!(f, File { name: "test/test.txt", data: "test data".into() });
    }

    #[test]
    #[should_panic]
    fn file_from_path_nonexistent() {
        File::from_path("nonexistent");
    }

    #[test]
    fn file_name() {
        let f = File { name: "f", data: "abc".into() };
        assert_eq!(f.name(), "f");
    }

    #[test]
    fn file_data() {
        let f = File { name: "f", data: "abc".into() };
        assert_eq!(f.data(), "abc");
    }

    #[test]
    fn file_fmt_display() {
        let f = File { name: "f", data: "abc".into() };
        let s = format!("{}", &f);
        assert_eq!(s, "f");
    }

    #[test]
    fn file_fmt_debug() {
        let f = File { name: "f", data: "abc".into() };
        let s = format!("{:?}", &f);
        assert_eq!(s, "f(3)");
    }

    // -------------------------------------------------------------------------

    #[test]
    fn pos_bof() {
        let p = Pos::bof();
        assert_eq!(p, Pos { byte: 0, line: 1, column: 1 });
    }

    #[test]
    fn pos_advance() {
        let mut p = Pos::bof();
        p.advance('a');
        assert_eq!(p, Pos { byte: 1, line: 1, column: 2 });
        p.advance('\u{10ABCD}');
        assert_eq!(p, Pos { byte: 5, line: 1, column: 3 });
    }

    #[test]
    fn pos_newline() {
        let mut p = Pos::bof();
        p.advance('\n');
        p.newline();
        assert_eq!(p, Pos { byte: 1, line: 2, column: 1 });
    }

    #[test]
    fn pos_fmt_display() {
        let p = Pos { byte: 1, line: 2, column: 3 };
        let s = format!("{}", &p);
        assert_eq!(s, "2:3");
    }

    #[test]
    fn pos_fmt_debug() {
        let p = Pos { byte: 1, line: 2, column: 3 };
        let s = format!("{:?}", &p);
        assert_eq!(s, "2:3[1]");
    }
}

