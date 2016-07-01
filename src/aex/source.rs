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

use std::fmt;
use std::fs;
use std::io::{self, Read};

use aex::pos::Pos;

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Source<'a> {
    // Intrinsic language feature
    BuiltIn,

    // Provided by source file
    File {
        pos: &'a Pos<'a>,   // position within file
        len: usize          // length, in bytes
    }
}

impl<'a> fmt::Display for Source<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Source::BuiltIn           => f.write_str("(built-in)"),
            Source::File { pos, len } => (pos as &fmt::Display).fmt(f),
        }
    }
}

// -----------------------------------------------------------------------------

pub struct File<'a> {
    pub name: &'a str,
    pub data: String,
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
}

fn fail_read(name: &str, error: io::Error) -> ! {
    panic!("error reading '{}': {}", name, error)
}

// -----------------------------------------------------------------------------

#[cfg(test)]
pub mod tests {
    use super::*;
    use aex::pos::Pos;

    pub static BOF: Source<'static> = Source::File {
        pos: &Pos { file: "f", byte: 0, line: 1, column: 1 },
        len: 0
    };
}

