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
//use std::ops::Range;
use std::ptr::copy_nonoverlapping;

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
    /// Reads `n` bytes from the stream.
    fn read_exact(&mut self, n: usize) -> Result<&[u8]>;

    /*
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
    */
}

#[derive(Debug)]
pub struct DecodeCursor<R: Read> {
    //
    //           __________________ rewind space, right aligned
    //           |                    unconsumed bytes are moved here before
    //           |   FETCH_IDX        fetching the next chunk from the source
    //           |   |
    //           |   |      _______ fetch space, left aligned
    //           |   |      |         chunk of bytes fetched from the source
    //           |   |      |
    //       |<----->|<------------>|
    // buf:  ....rrrrFFFFFFFFFFFF....
    //           |       |       |_ tail
    //           |       |_________ idx
    //           |_________________ head
    //
    buf:  Box<[u8]>,    // buffer: rewind space + fetch space
    idx:  usize,        // buffer index at next readable byte
    tail: usize,        // buffer index after last fetched byte
    head: usize,        // buffer index    at first byte of current read op
    pos:  u64,          // stream position at first byte of current read op
    src:  R,            // source stream
}

static EMPTY: [u8; 0] = [];

const REWIND_CAP: usize =      256; // bytes
const FETCH_SIZE: usize = 8 * 1024; // bytes
const FETCH_IDX:  usize = REWIND_CAP;
const BUF_SIZE:   usize = REWIND_CAP + FETCH_SIZE;

impl<R: Read> DecodeCursor<R> {
    pub fn new(src: R) -> Self {
        Self {
            buf:   Box::new([0; BUF_SIZE]),
            idx:   FETCH_IDX,
            tail:  FETCH_IDX,
            head:  FETCH_IDX,
            pos:   0,
            src:   src,
        }
    }

    // Check if there are unconsumed (rewindable) bytes.  If so, shift them
    // to the rewind space to make room for the incoming chunk.
    //
    fn shift_unconsumed(&mut self) {
        assert!(FETCH_IDX <= self.head);
        assert!(self.head <= self.idx);
        assert!(self.idx  <= self.tail);
        assert!(self.tail == BUF_SIZE);

        let unconsumed = self.tail - self.head;

        if unconsumed == 0 {
            self.head = FETCH_IDX;
            self.idx  = FETCH_IDX;
            self.tail = FETCH_IDX;
        } else {
            assert!(unconsumed <= REWIND_CAP);

            let src = self.buf[self.head..self.tail].as_ptr() as *const u8;

            let unread = self.tail - self.idx;
            self.head  = FETCH_IDX - unconsumed;
            self.idx   = FETCH_IDX - unread;
            self.tail  = FETCH_IDX;

            let dst = self.buf[self.head..self.tail].as_ptr() as *mut u8;

            unsafe { copy_nonoverlapping(src, dst, unconsumed); }
        }
    }

    // Fill the fetch space from the source stream.
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
                    // Read some bytes, possibly not all requested
                    let tmp   = buf;
                        buf   = &mut tmp[n..];
                        tail += n;

                    if buf.is_empty() {
                        // Read all requested bytes
                        break;
                    }
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

impl<R: Read> ReadAhead for DecodeCursor<R> {
    /// Reads exactly `n` bytes from the stream.
    ///
    /// Returns a slice containing the read bytes.
    ///
    fn read_exact(&mut self, n: usize) -> Result<&[u8]> {
        assert!(self.head <= self.idx);
        assert!(self.idx  <= self.tail);
        assert!(FETCH_IDX <= self.tail);
        assert!(self.tail <= BUF_SIZE);

        let mut beg = self.idx;
        let mut end = beg + n;

        // Disallow a read causing unconsumed bytes to exceed rewind capacity
        if end - self.head > REWIND_CAP {
            return Err(E::new(Other, "rewind capacity exceeded"));
        }

        // Handle when buffer has too few bytes
        if end > self.tail {
            // Read might have requested more bytes than supported
            if n > FETCH_SIZE {
                return Err(E::new(InvalidInput, "read size exceeded internal buffer size"));
            }

            // Prior read might have reached end-of-file
            if self.tail < BUF_SIZE {
                return Err(E::new(UnexpectedEof, "unexpected end-of-file"));
            }

            // No problems, just buffer exhausted
            self.shift_unconsumed();
            self.fetch()?;

            // Fetch invalidates the previous buffer indexes
            beg = self.idx;
            end = beg + n;

            // Fetch might have reached end-of-file
            if end > self.tail {
                // Not enough bytes to satisfy the read request
                // OR: Read request bigger than our buffer! TODO
                return Err(E::new(UnexpectedEof, "unexpected end-of-file"));
            }
        }

        // Return a view into the buffer
        Ok(&self.buf[beg..end])
    }

    /*
    /// Returns the position of the first byte in the current read-ahead.
    #[inline]
    fn start(&self) -> u64 { self.start }

    /// Returns the count of bytes in the current read-ahead.
    #[inline]
    fn len(&self) -> u64 { self.more.position() }

    #[inline]
    fn get(&mut self, range: Range<usize>) -> &[u8] {
      //self.len = cmp::max(self.len, range.end);
        &self.more.get_ref()[range]
    }

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
    */
}

