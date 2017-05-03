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
    vec:     Vec<u8>,   // pending bytes + unread rewound bytes
    src:     R,         // source stream
    vec_pos: usize,     // position in vec of next read; pending byte count
    src_pos: usize,     // position in src of vec[0]; consumed byte count
}

const BUF_CAP: usize = 64;

impl<R: ReadToBuf> DecodeCursor<R> {
    pub fn new(src: R) -> Self {
        Self { vec: vec![0; BUF_CAP], src, vec_pos: 0, src_pos: 0 }
    }

    #[inline]
    pub fn consumed_pos(&self) -> usize {
        self.src_pos
    }

    #[inline]
    pub fn pending_len(&self) -> usize {
        self.vec_pos
    }

    #[inline]
    pub fn pending_pos(&self) -> usize {
        self.src_pos + self.vec_pos
    }

    #[inline]
    pub fn pending_bytes(&self) -> &[u8] {
        &self.vec[..self.vec_pos]
    }

    pub fn consume(&mut self) {
        // Consider pending bytes consumed
        self.src_pos += self.vec_pos;
        self.vec_pos  = 0;

        // Forget pending bytes
        self.vec.drain(..self.vec_pos);
    }

    pub fn read_bytes(&mut self, n: usize) -> Result<&[u8]> {
        // Compute vector indexes of the read
        let beg = self.vec_pos;
        let end = match beg.checked_add(n) {
            Some(n) => n,
            None    => panic!("read_bytes: would overflow buffer"),
        };

        // Compute bytes needed from source vs. already buffered in vector
        let have = self.vec.len();
        let need = end.saturating_sub(have);

        // Read bytes from source, if required
        if need != 0 {
            // Enlarge vector to make room for the read
            self.vec.resize(end, 0);

            // Read into the new vector space
            let read = self.src.read_to_buf(&mut self.vec[have..end])?;

            // Handle short read
            if read != need {
                self.vec.truncate(have + read);
                return Err(E::new(UnexpectedEof, "failed to read requested bytes"));
            }
        }

        // Consider bytes pending
        self.vec_pos += end - beg;

        // Return view into the vector
        Ok(&self.vec[beg..end])
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;

    #[test]
    pub fn read_bytes() {
        let mut c = cursor();

        let bytes = c.read_bytes(2).unwrap();

        assert_eq!(bytes.len(), 2, "sdfsdf");
    }

    fn cursor() -> DecodeCursor<Cursor<Vec<u8>>> {
        let nums = (0..).take(BUF_CAP * 2 + 2).map(|n| n as u8).collect();
        let src  = Cursor::new(nums);
        DecodeCursor::new(src)
    }
}

