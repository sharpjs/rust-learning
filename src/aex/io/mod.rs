// Input/Output Extensions
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

use std::io::{Read, Result};
use std::io::ErrorKind::*;

mod rewind_read;
pub use self::rewind_read::*;

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

