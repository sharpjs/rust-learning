// Input/Output Helpers
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

use std::io::{/*self,*/ BufRead, Cursor};
use std::ops::Range;

/// A forward-only, read-only cursor with rewindable read-ahead.
///
/// Read bytes remain unconsumed until `consume` is called, upon which the
/// bytes become irrevocably consumed.  If `rewind` is called instead, the
/// cursor resets to the first unconsumed position, and the unconsumed bytes
/// can be read again.
///   
///     consumed   unconsumed   future
///     ...........*************????????>
///                |<---------->|
///                |      |     |_ end
///                |      |_______ len
///                |______________ start
///
pub trait ReadAhead {
    /// Returns the position of the first byte in the current read-ahead.
    fn start(&self) -> u64;

    /// Returns the count of bytes in the current read-ahead.
    fn len(&self) -> u64;

    /// Returns the position of the first unread byte.
    fn end(&self) -> u64 {
        self.start() + self.len()
    }

    /// Consumes the current read-ahead and advances the cursor to the first
    /// unread byte.
    ///
    /// * `start` advances by `len` and becomes equal to `end`.
    /// * `end` remains unchanged.
    /// * `len` becomes `0`.
    ///
    fn consume(&mut self);

    /// Forgets the current read-ahead and rewinds the cursor to the first
    /// unconsumed byte.
    ///
    /// * `start` remains unchanged.
    /// * `end` rewinds by `len` and becomes equal to `start`.
    /// * `len` becomes `0`.
    ///
    fn rewind(&mut self);
}

#[derive(Debug)]
pub struct BufReadAhead<'a, R: BufRead + 'a> {
    more:  Cursor<&'a [u8]>,    // current read-ahead
    src:   &'a mut R,           // source stream
    start: u64,                 // byte position of first read-ahead byte
}

static EMPTY: [u8; 0] = [];

impl<'a, R: BufRead> BufReadAhead<'a, R> {
    pub fn new(src: &'a mut R, pos: u64) -> Self {
        Self {
            more:  Cursor::new(&EMPTY),
            src:   src,
            start: pos,
        }
    }
}

impl<'a, R: BufRead> ReadAhead for BufReadAhead<'a, R> {
    /// Returns the position of the first byte in the current read-ahead.
    #[inline]
    fn start(&self) -> u64 { self.start }

    /// Returns the count of bytes in the current read-ahead.
    #[inline]
    fn len(&self) -> u64 { self.more.position() }

 // #[inline]
 // fn get(&mut self, range: Range<usize>) -> &[u8] {
 //   //self.len = cmp::max(self.len, range.end);
 //     &self.more.get_ref()[range]
 // }

    /// Consumes the current read-ahead and advances the cursor to the first
    /// unread byte.
    ///
    /// * `start` advances by `len` and becomes equal to `end`.
    /// * `end` remains unchanged.
    /// * `len` becomes `0`.
    ///
    fn consume(&mut self) { panic!() }

    /// Forgets the current read-ahead and rewinds the cursor to the first
    /// unconsumed byte.
    ///
    /// * `start` remains unchanged.
    /// * `end` rewinds by `len` and becomes equal to `start`.
    /// * `len` becomes `0`.
    ///
    fn rewind(&mut self) { panic!() }
}

