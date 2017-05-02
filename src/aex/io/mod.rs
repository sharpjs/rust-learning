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

/// TODO
///
#[derive(Debug)]
pub struct DecodeCursor<R: ReadToBuf> {
    vec: Vec<u8>,
    src: R,
}

impl<R: ReadToBuf> DecodeCursor<R> {
    pub fn read_bytes(&mut self, n: usize) -> Result<&[u8]> {
        // Enlarge vector to make room for the read
        let beg = self.vec.len();
        let end = beg + n;
        self.vec.resize(end, 0);

        // Read into the new vector elements
        let actual = self.src.read_to_buf(&mut self.vec[beg..end])?;

        // Handle short read
        if actual != n {
            let unread = n - actual;
            self.vec.resize(end - unread, 0);
            return Err(E::new(UnexpectedEof, "failed to read requested bytes"));
        }

        // Return view into the vector
        Ok(&self.vec[beg..end])
    }
}

