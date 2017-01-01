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

use std::cmp::Ordering;
use std::fmt::{self, Debug, Display, Formatter};
use std::fs;
use std::io::{self, Read};
use std::ops::BitOr;

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

// For Source, "bitwise or" means union of positions.
//
impl<'a> BitOr for Source<'a> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Source::File { file: lf, pos: lp, len: ln },
             Source::File { file: rf, pos: rp, len: rn }) => {
                if lf != rf {
                    panic!("Cannot compute union of positions from multiple files.")
                } else if lp < rp {
                    Source::File { file: lf, pos: lp, len: rp.byte + rn - lp.byte }
                } else {
                    Source::File { file: rf, pos: rp, len: lp.byte + ln - rp.byte }
                }
            },
            (Source::BuiltIn, rhs) => rhs,
            (lhs,             _  ) => lhs
        }
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
    #[inline]
    pub fn new<D: Into<String>>(name: &'a str, data: D) -> Self {
        File { name: name, data: data.into() }
    }

    pub fn from_reader<R: Read>(name: &'a str, mut reader: R) -> Self {
        let mut data = String::new();

        match reader.read_to_string(&mut data) {
            Ok  (_) => Self::new(name, data),
            Err (e) => fail_read(name, e)
        }
    }

    pub fn from_stdin() -> Self {
        Self::from_reader("(stdin)", io::stdin())
    }

    pub fn from_path(path: &'a str) -> Self {
        match fs::File::open(path) {
            Ok  (f) => Self::from_reader(path, f),
            Err (e) => fail_read(path, e)
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

impl PartialOrd for Pos {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pos {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.byte.cmp(&other.byte)
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

    fn with_source<F: Fn(&Source)>(f: F) {
        let file = File { name: "f", data: "abc".into() };
        let src  = Source::File { file: &file, pos: Pos::bof(), len: 2 };
        f(&src);
    }

    #[test]
    fn source_text() {
        with_source(|s| assert_eq!(s.as_ref(), "ab"));
    }

    #[test]
    fn source_fmt_display() {
        with_source(|s| assert_eq!(format!("{}", s), "f:1:1"));
    }

    #[test]
    fn source_fmt_debug() {
        with_source(|s| assert_eq!(format!("{:?}", s), "f(3):1:1[0]+2"));
    }

    #[test]
    fn source_union() {
        let f = File { name: "f", data: "abcdefghijklmnop".into() };

        let     pa = Pos::bof();
        let mut pb = pa; pb.advance('_'); pb.advance('_'); pb.advance('_');

        let sa = Source::File { file: &f, pos: pa, len: 2 };
        let sb = Source::File { file: &f, pos: pb, len: 2 };
        let ex = Source::File { file: &f, pos: pa, len: 5 };

        assert_eq!(sa | sb, ex);
        assert_eq!(sb | sa, ex);
    }

    #[test]
    #[should_panic]
    fn source_union_multiple_files() {
        let fa = File { name: "fa", data: "ab".into() };
        let fb = File { name: "fb", data: "ab".into() };
        let p  = Pos::bof();

        let sa = Source::File { file: &fa, pos: p, len: 2 };
        let sb = Source::File { file: &fb, pos: p, len: 2 };

        sa | sb;
    }

    // -------------------------------------------------------------------------

    fn with_file<F: Fn(&File)>(f: F) {
        let file = File { name: "f", data: "abc".into() };
        f(&file);
    }

    #[test]
    fn file_new() {
        let f = File::new("f", "abc");
        assert_eq!(f, File { name: "f", data: "abc".into() });
    }

    #[test]
    fn file_from_reader() {
        let c = Cursor::new("abc");
        let f = File::from_reader("f", c);
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
        with_file(|f| assert_eq!(f.name(), "f"));
    }

    #[test]
    fn file_data() {
        with_file(|f| assert_eq!(f.data(), "abc"));
    }

    #[test]
    fn file_fmt_display() {
        with_file(|f| assert_eq!(format!("{}", &f), "f"));
    }

    #[test]
    fn file_fmt_debug() {
        with_file(|f| assert_eq!(format!("{:?}", &f), "f(3)"));
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
    fn pos_compare() {
        let mut a = Pos::bof(); a.advance('a');
        let mut b = Pos::bof(); b.advance('b'); b.advance('b');
        assert!(a  < b);
        assert!(b  > a);
        assert!(a == a);
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

