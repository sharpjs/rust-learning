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

use std::io::{Error as E, Read, Result};
use std::io::ErrorKind::*;
use std::mem::size_of;
use std::ptr::copy;

macro_rules! read_int {
    ($r:expr, $t:ty, $f:ident) => {{
        let n = size_of::<$t>();
        let p = $r.read_exact(n)?.as_ptr() as *const $t;
        let v = unsafe { *p };
        Ok(v.$f())
    }};
}

/// Reader interface suitable for instruction decodding.
///
///                rewindable/
///     consumed   unconsumed    unread
///     ...........**************????????>
///                |<----------->|
///                |      |      |__ end
///                |      |_________ len
///                |________________ start
///
pub trait DecodeRead {
    /// Reads exactly `n` bytes from the stream.
    ///
    /// Returns a slice containing the read bytes.
    ///
    fn read_exact(&mut self, n: usize) -> Result<&[u8]>;

    /// Consumes any rewindable bytes and moves the rewind bookmark to the
    /// current reader position.
    ///
    /// * `start` advances by `len` and becomes equal to `end`.
    /// * `end` remains unchanged.
    /// * `len` becomes `0`.
    ///
    fn consume(&mut self);

    /// Forgets the current read-ahead and rewinds the reader to the first
    /// unconsumed byte.
    ///
    /// * `start` remains unchanged.
    /// * `end` rewinds by `len` and becomes equal to `start`.
    /// * `len` becomes `0`.
    ///
    fn rewind(&mut self);

    /// Returns the position of the first byte in the current read-ahead.
    fn start(&self) -> u64;

    /// Returns the count of bytes in the current read-ahead.
    fn len(&self) -> u64;

    /// Returns the position of the first unread byte.
    fn end(&self) -> u64 {
        self.start() + self.len()
    }

    fn read_u8(&mut self) -> Result<u8> {
        unsafe { Ok(*self.read_exact(1)?.get_unchecked(0)) }
    }

    fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    fn read_u16_be(&mut self) -> Result<u16> {
        read_int!(self, u16, to_be)
    }

    fn read_u32_be(&mut self) -> Result<u32> {
        read_int!(self, u32, to_be)
    }

    fn read_u64_be(&mut self) -> Result<u64> {
        read_int!(self, u64, to_be)
    }

    fn read_i16_be(&mut self) -> Result<i16> {
        Ok(self.read_u16_be()? as i16)
    }

    fn read_i32_be(&mut self) -> Result<i32> {
        Ok(self.read_u32_be()? as i32)
    }

    fn read_i64_be(&mut self) -> Result<i64> {
        Ok(self.read_u64_be()? as i64)
    }

    fn read_u16_le(&mut self) -> Result<u16> {
        read_int!(self, u16, to_le)
    }

    fn read_u32_le(&mut self) -> Result<u32> {
        read_int!(self, u32, to_le)
    }

    fn read_u64_le(&mut self) -> Result<u64> {
        read_int!(self, u64, to_le)
    }

    fn read_i16_le(&mut self) -> Result<i16> {
        Ok(self.read_u16_le()? as i16)
    }

    fn read_i32_le(&mut self) -> Result<i32> {
        Ok(self.read_u32_le()? as i32)
    }

    fn read_i64_le(&mut self) -> Result<i64> {
        Ok(self.read_u64_le()? as i64)
    }
}

/// Adapts any reader to an interface suitable for decoding.
#[derive(Debug)]
pub struct DecodeReader<R: Read> {
    //
    //           ___________________ carryover space, right aligned
    //           |                     rewindable or unread bytes are moved
    //           |                     here before fetching more bytes from
    //           |                     the source
    //           |   FETCH_IDX
    //           |   |       _______ fetch space, left aligned
    //           |   |       |         bytes fetched from the source are
    //           |   |       |         placed here
    //           |   |       |
    //       0   |   |       |       BUF_SIZE
    //       |   |   |       |       |
    //       |<----->|<------------->|
    // buf:  ....rrrrFFFFFFFFFFFF....
    //           |<----->|<----->|
    //           |   |   |   |   |____ tail
    //           |   |   |   |________   unread bytes
    //           |   |   |____________ idx
    //           |   |________________   rewindable bytes
    //           |____________________ head
    //
    buf:  Box<[u8]>,    // buffer: carryover space + fetch space
    head: usize,        // buffer index of rewind bookmark
    idx:  usize,        // buffer index of next readable byte
    tail: usize,        // buffer index after last fetched byte
    pos:  u64,          // source stream position of rewind bookmark
    src:  R,            // source stream
}

const REWIND_CAP: usize =      256; // bytes
const FETCH_SIZE: usize = 8 * 1024; // bytes
const FETCH_IDX:  usize = REWIND_CAP;
const BUF_SIZE:   usize = REWIND_CAP + FETCH_SIZE;

impl<R: Read> DecodeReader<R> {
    /// Creates a new reader for the given source.
    pub fn new(src: R) -> Self {
        Self {
            buf:  Box::new([0; BUF_SIZE]),
            idx:  FETCH_IDX,
            tail: FETCH_IDX,
            head: FETCH_IDX,
            pos:  0,
            src:  src,
        }
    }

    /// Shifts rewindable and unread bytes into the carryover space, so that
    /// the fetch space is ready to accept another fetch.
    fn shift_carryover(&mut self) {
        debug_assert!(self.head <= self.idx);
        debug_assert!(self.idx  <= self.tail);
        debug_assert!(FETCH_IDX <= self.tail);
        debug_assert!(self.tail <= BUF_SIZE);

        // Bytes might be completely within the carryover space already.
        // In that case, there is nothing to do.
        if self.tail == FETCH_IDX { return; }

        match self.tail - self.head {
            0 => {
                // No bytes need to be shifted, but buffer indexes still must
                // be normalized in preparation for an upcoming fetch.
                self.head = FETCH_IDX;
                self.idx  = FETCH_IDX;
                self.tail = FETCH_IDX;
            },
            carryover => {
                debug_assert!(carryover <= REWIND_CAP);

                let src = self.buf[self.head..self.tail].as_ptr() as *const u8;

                let unread = self.tail - self.idx;
                self.head  = FETCH_IDX - carryover;
                self.idx   = FETCH_IDX - unread;
                self.tail  = FETCH_IDX;

                let dst = self.buf[self.head..self.tail].as_ptr() as *mut u8;

                unsafe { copy(src, dst, carryover); }
            },
        }
    }

    /// Fills the fetch space from the source stream.
    fn fetch(&mut self) -> Result<()> {
        assert!(self.tail == FETCH_IDX);

        let     src  = &mut self.src;
        let mut buf  = &mut self.buf[FETCH_IDX..];
        let mut tail = FETCH_IDX;

        loop {
            match src.read(buf) {
                Ok(0) => {
                    // Reached end of stream
                    break;
                },
                Ok(n) => {
                    // Got some bytes, possibly not all requested
                    let tmp = buf;
                    buf = &mut tmp[n..];
                    tail += n;

                    // Check if got all requested bytes
                    if buf.is_empty() { break; }
                },
                Err(ref e) if e.kind() == Interrupted => {
                    // Read interrupted; retry
                    continue;
                },
                Err(e) => return Err(e)
            }
        }

        self.tail = tail;
        Ok(())
    }
}

impl<R: Read> DecodeRead for DecodeReader<R> {
    /// Reads exactly `n` bytes.
    fn read_exact(&mut self, n: usize) -> Result<&[u8]> {
        assert!(self.head <= self.idx);
        assert!(self.idx  <= self.tail);
        assert!(FETCH_IDX <= self.tail);
        assert!(self.tail <= BUF_SIZE);

        // Compute read range
        let mut beg = self.idx;
        let mut end = beg.saturating_add(n);

        // Verify that read would not cause rewindable bytes to exceed capacity
        if end - self.head > REWIND_CAP {
            return Err(E::new(Other, "rewind capacity exceeded"));
        }

        // Check if buffer has too few bytes to fulfill read
        if end > self.tail {
            // Shift rewindable and unread bytes into carryover space to make
            // room for a fetch.  Verified above to be <=REWIND_CAP bytes.
            self.shift_carryover();

            // Shifting changes indexes; recompute read range
            beg = self.idx;
            end = beg + n;

            // Try to get more bytes
            self.fetch()?;

            // Fetch might have reached end-of-file
            if end > self.tail {
                return Err(E::new(UnexpectedEof, "unexpected end-of-file"));
            }
        }

        // Return a view into the buffer
        self.idx = end;
        Ok(&self.buf[beg..end])
    }

    /// Consumes any rewindable bytes and moves the rewind bookmark to the
    /// current reader position.
    ///
    /// * `start` advances by `len` and becomes equal to `end`.
    /// * `end` remains unchanged.
    /// * `len` becomes `0`.
    ///
    fn consume(&mut self) {
        self.pos += self.len();
        self.head = self.idx;
    }

    /// Forgets the current read-ahead and rewinds the reader to the first
    /// unconsumed byte.
    ///
    /// * `start` remains unchanged.
    /// * `end` rewinds by `len` and becomes equal to `start`.
    /// * `len` becomes `0`.
    ///
    #[inline]
    fn rewind(&mut self) {
        self.idx = self.head;
    }

    /// Returns the position of the first byte in the current read-ahead.
    #[inline]
    fn start(&self) -> u64 {
        self.pos
    }

    /// Returns the count of bytes in the current read-ahead.
    #[inline]
    fn len(&self) -> u64 {
        (self.idx - self.head) as u64
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::io::ErrorKind::*;
    use super::*;

    #[test]
    fn initial() {
        let c = reader();

        assert_eq!(c.end(),   0);
        assert_eq!(c.start(), 0);
        assert_eq!(c.len(),   0);
    }

    #[test]
    fn read_oversized() {
        let mut c = reader();

        {
            let result = c.read_exact(BUF_SIZE + 1);

            assert!(result.is_err());
        }

        // After error, indexes should be unchanged
        assert_eq!(c.start(), 0);
        assert_eq!(c.len(),   0);
        assert_eq!(c.end(),   0);
    }

    #[test]
    fn read_once() {
        let mut c = reader();

        {
            let bytes = c.read_exact(2).unwrap();

            assert_eq!(bytes.len(), 2);
            assert_eq!(bytes[0], 0);
            assert_eq!(bytes[1], 1);
        }

        assert_eq!(c.start(), 0);
        assert_eq!(c.len(),   2);
        assert_eq!(c.end(),   2);
    }

    #[test]
    fn read_and_consume() {
        let mut c = reader();

        {
            let bytes = c.read_exact(2).unwrap();

            assert_eq!(bytes.len(), 2);
            assert_eq!(bytes[0], 0);
            assert_eq!(bytes[1], 1);
        }

        c.consume();

        assert_eq!(c.start(), 2);
        assert_eq!(c.len(),   0);
        assert_eq!(c.end(),   2);

        {
            let bytes = c.read_exact(2).unwrap();

            assert_eq!(bytes.len(), 2);
            assert_eq!(bytes[0], 2);
            assert_eq!(bytes[1], 3);
        }

        assert_eq!(c.start(), 2);
        assert_eq!(c.len(),   2);
        assert_eq!(c.end(),   4);
    }

    #[test]
    fn read_and_rewind() {
        let mut c = reader();

        {
            let bytes = c.read_exact(2).unwrap();

            assert_eq!(bytes.len(), 2);
            assert_eq!(bytes[0], 0);
            assert_eq!(bytes[1], 1);
        }

        c.rewind();

        assert_eq!(c.end(),   0);
        assert_eq!(c.start(), 0);
        assert_eq!(c.len(),   0);

        {
            let bytes = c.read_exact(2).unwrap();

            assert_eq!(bytes.len(), 2);
            assert_eq!(bytes[0], 0);
            assert_eq!(bytes[1], 1);
        }

        assert_eq!(c.start(), 0);
        assert_eq!(c.len(),   2);
        assert_eq!(c.end(),   2);
    }

    #[test]
    fn read_until_rewind_exceeded() {
        let mut c = reader();

        c.read_exact(REWIND_CAP).unwrap();
        let result = c.read_exact(1);

        assert!(result.is_err());
        assert_eq!(result.err().unwrap().kind(), Other);
    }

    #[test]
    fn read_until_eof() {
        let mut c = reader();
        let mut n = 0;

        loop {
            match c.read_exact(3) {
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

    #[test]
    fn read_u8() {
        assert_eq!(reader().read_u8().unwrap(), 0x00);
    }

    #[test]
    fn read_i8() {
        assert_eq!(reader().read_i8().unwrap(), 0x00);
    }

    #[test]
    fn read_u16_be() {
        assert_eq!(reader().read_u16_be().unwrap(), 0x0001);
    }

    #[test]
    fn read_u32_be() {
        assert_eq!(reader().read_u32_be().unwrap(), 0x00010203);
    }

    #[test]
    fn read_u64_be() {
        assert_eq!(reader().read_u64_be().unwrap(), 0x0001020304050607);
    }

    #[test]
    fn read_i16_be() {
        assert_eq!(reader().read_i16_be().unwrap(), 0x0001);
    }

    #[test]
    fn read_i32_be() {
        assert_eq!(reader().read_i32_be().unwrap(), 0x00010203);
    }

    #[test]
    fn read_i64_be() {
        assert_eq!(reader().read_i64_be().unwrap(), 0x0001020304050607);
    }

    #[test]
    fn read_u16_le() {
        assert_eq!(reader().read_u16_le().unwrap(), 0x0100);
    }

    #[test]
    fn read_u32_le() {
        assert_eq!(reader().read_u32_le().unwrap(), 0x03020100);
    }

    #[test]
    fn read_u64_le() {
        assert_eq!(reader().read_u64_le().unwrap(), 0x0706050403020100);
    }

    #[test]
    fn read_i16_le() {
        assert_eq!(reader().read_i16_le().unwrap(), 0x0100);
    }

    #[test]
    fn read_i32_le() {
        assert_eq!(reader().read_i32_le().unwrap(), 0x03020100);
    }

    #[test]
    fn read_i64_le() {
        assert_eq!(reader().read_i64_le().unwrap(), 0x0706050403020100);
    }

    fn reader() -> DecodeReader<Cursor<Vec<u8>>> {
        let nums = (0..).take(17000).map(|n| n as u8).collect();
        let src  = Cursor::new(nums);
        DecodeReader::new(src)
    }
}

