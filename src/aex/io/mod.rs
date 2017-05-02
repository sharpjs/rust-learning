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

//mod decode;
//mod endian;

//pub use self::decode::*;
//pub use self::endian::*;

use std::io::{Error as E, Read, Result};
use std::io::ErrorKind::*;

/// Extends `std::io::Read` with the `read_to_buf` method.
pub trait ReadToBuf: Read {
    /// Read enough bytes to fill `buf`, returning how many bytes were read.
    ///
    /// This function is equivalent to `std::io::Read::read_exact()` except for
    /// its behavior at end of file.  If this function encounters EOF before
    /// completely filling `buf`, it returns the number of bytes read, which
    /// will be less than the requested amount.
    ///
    fn read_to_buf(&mut self, mut buf: &mut [u8]) -> Result<usize> {
        let mut total = 0;

        while !buf.is_empty() {
            let n = match self.read(buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(ref e) if e.kind() == Interrupted => continue,
                Err(e) => return Err(e),
            };

            total += n;

            let tmp = buf;
            buf = &mut tmp[n..];
        }

        Ok(total)
    }
}

impl<R: Read> ReadToBuf for R { }

/// TODO
///
#[derive(Debug)]
pub struct DecodeCursor<R: ReadToBuf> {
    vec: Vec<u8>,
    src: R,
    len: usize, // length of virtually-read elements
    pos: usize, // pos in src of vec[0]
}

impl<R: ReadToBuf> DecodeCursor<R> {
    pub fn new(src: R) -> Self {
        Self { vec: vec![0; 64], src, len: 0, pos: 0 }
    }

    pub fn current_read(&self) -> &[u8] {
        &self.vec[..self.len]
    }

    pub fn current_read_len(&self) -> usize {
        self.len
    }

    pub fn start_pos(&self) -> usize {
        self.pos
    }

    pub fn end_pos(&self) -> usize {
        self.pos + self.len
    }

    pub fn consume(&mut self) {
        self.pos += self.len;
        self.vec.drain(..self.len);
    }

    pub fn read_bytes(&mut self, n: usize) -> Result<&[u8]> {
        // Enlarge vector to make room for the read
        //
        let beg  = self.len;
        let end  = beg + n;

        let have = self.vec.len();
        let need = end - have;

        if need > 0 {
            self.vec.reserve(need);

            // Read into the new vector elements
            let read = self.src.read_to_buf(&mut self.vec[have..end])?;

            // Handle short read
            if read != need {
                self.vec.truncate(have + read);
                return Err(E::new(UnexpectedEof, "failed to read requested bytes"));
            }
        }

        // Return view into the vector
        Ok(&self.vec[beg..end])
    }
}

