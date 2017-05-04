// A Rewindable Reader
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

use std::io::{Error as E, Result};
use std::io::ErrorKind::*;

use super::ReadToBuf;

/// A reader that can rewind to an earlier position in the stream.
pub trait RewindRead {
    /// Returns the position of the reader, considering only consumed bytes.
    fn consumed_pos(&self) -> usize;

    /// Returns the count of pending (rewindable) bytes of the reader.
    fn pending_len(&self) -> usize;

    /// Returns the position of the reader, considering both consumed and pending bytes.
    fn pending_pos(&self) -> usize;

    /// Returns the pending bytes of the reader as a slice.
    fn pending_bytes(&self) -> &[u8];

    /// Consumes pending bytes and marks the rewind point.
    fn consume(&mut self);

    /// Rewinds pending bytes, making them readable again.
    fn rewind(&mut self);

    /// Reads `n` bytes, returning them as a slice.
    fn read_bytes(&mut self, n: usize) -> Result<&[u8]>;
}

/// Transforms any reader into a rewindable reader.
#[derive(Clone, Debug)]
pub struct RewindReader<R: ReadToBuf> {
    vec:     Vec<u8>,   // rewindable bytes + unread rewound bytes
    src:     R,         // source stream
    vec_pos: usize,     // position in vec of next read; rewindable byte count
    src_pos: usize,     // position in src of vec[0]; consumed byte count
}

const BUF_CAP: usize = 64;

impl<R: ReadToBuf> RewindReader<R> {
    pub fn new(src: R) -> Self {
        Self { vec: Vec::with_capacity(BUF_CAP), src, vec_pos: 0, src_pos: 0 }
    }
}

impl<R: ReadToBuf> RewindRead for RewindReader<R> {
    #[inline]
    fn consumed_pos(&self) -> usize {
        self.src_pos
    }

    #[inline]
    fn pending_len(&self) -> usize {
        self.vec_pos
    }

    #[inline]
    fn pending_pos(&self) -> usize {
        self.src_pos + self.vec_pos
    }

    #[inline]
    fn pending_bytes(&self) -> &[u8] {
        &self.vec[..self.vec_pos]
    }

    fn consume(&mut self) {
        // Forget rewindable bytes
        self.vec.drain(..self.vec_pos);

        // Consider rewindable bytes consumed
        self.src_pos += self.vec_pos;
        self.vec_pos  = 0;
    }

    #[inline]
    fn rewind(&mut self) {
        // Make all rewindable bytes unread again
        self.vec_pos = 0;
    }

    fn read_bytes(&mut self, n: usize) -> Result<&[u8]> {
        // Compute vector indexes of the read
        let beg = self.vec_pos;
        let end = match beg.checked_add(n) {
            Some(x) => x,
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

        // Make these bytes rewindable
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
    fn read_bytes() {
        let mut c = cursor();

        let bytes = c.read_bytes(2).unwrap();

        assert_eq!(bytes.len(), 2, "sdfsdf");
    }

    #[test]
    fn read_once() {
        let mut c = cursor();

        {
            let bytes = c.read_bytes(2).unwrap();

            assert_eq!(bytes.len(), 2);
            assert_eq!(bytes[0],    0);
            assert_eq!(bytes[1],    1);
        }

        assert_eq!(c.consumed_pos(), 0);
        assert_eq!(c.pending_len(),  2);
        assert_eq!(c.pending_pos(),  2);
    }

    #[test]
    fn read_and_consume() {
        let mut c = cursor();

        {
            let bytes = c.read_bytes(2).unwrap();

            assert_eq!(bytes.len(), 2);
            assert_eq!(bytes[0],    0);
            assert_eq!(bytes[1],    1);
        }

        c.consume();

        assert_eq!(c.consumed_pos(), 2);
        assert_eq!(c.pending_len(),  0);
        assert_eq!(c.pending_pos(),  2);

        {
            let bytes = c.read_bytes(2).unwrap();

            assert_eq!(bytes.len(), 2);
            assert_eq!(bytes[0],    2);
            assert_eq!(bytes[1],    3);
        }

        assert_eq!(c.consumed_pos(), 2);
        assert_eq!(c.pending_len(),  2);
        assert_eq!(c.pending_pos(),  4);
    }

    #[test]
    fn read_and_rewind() {
        let mut c = cursor();

        {
            let bytes = c.read_bytes(2).unwrap();

            assert_eq!(bytes.len(), 2);
            assert_eq!(bytes[0],    0);
            assert_eq!(bytes[1],    1);
        }

        c.rewind();

        assert_eq!(c.consumed_pos(), 0);
        assert_eq!(c.pending_pos(),  0);
        assert_eq!(c.pending_len(),  0);

        {
            let bytes = c.read_bytes(2).unwrap();

            assert_eq!(bytes.len(), 2);
            assert_eq!(bytes[0], 0);
            assert_eq!(bytes[1], 1);
        }

        assert_eq!(c.consumed_pos(), 0);
        assert_eq!(c.pending_len(),  2);
        assert_eq!(c.pending_pos(),  2);
    }

    #[test]
    fn read_until_eof() {
        let mut c = cursor();
        let mut n = 0;

        loop {
            match c.read_bytes(3) {
                Ok(bytes) => {
                    assert_eq!(bytes.len(), 3);
                    assert_eq!(bytes[0], (n * 3 + 0) as u8);
                    assert_eq!(bytes[1], (n * 3 + 1) as u8);
                    assert_eq!(bytes[2], (n * 3 + 2) as u8);
                },
                Err(ref e) => {
                    assert!(n > 0);
                    assert_eq!(e.kind(), UnexpectedEof);
                    break;
                }
            }

            c.consume();

            assert!(n < 17000);
            n += 1;
        }
    }

    fn cursor() -> RewindReader<Cursor<Vec<u8>>> {
        let nums = (0u8..).take(BUF_CAP * 2 + 2).collect();
        let src  = Cursor::new(nums);
        RewindReader::new(src)
    }
}
