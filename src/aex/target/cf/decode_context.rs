// Context for Instruction Decoding
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

use std::io::{self, BufRead, Cursor};
use byteorder::{BigEndian as BE, ReadBytesExt};

use super::Size;

#[derive(Debug)]
pub struct DecodeContext<'a, R: BufRead + 'a> {
    op_word: u16,               // operation word
    op_size: Size,              // operation size
    more:    Cursor<&'a [u8]>,  // extension words
    src:     &'a mut R,         // source stream
    offset:  u64,               // offset within stream
}

//#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
//pub enum Size { Byte, Word, Long }

static EMPTY: [u8; 0] = [];

impl<'a, R: BufRead> DecodeContext<'a, R> {
    pub fn new(src: &'a mut R, offset: u64) -> Self {
        DecodeContext {
            op_word: 0,
            op_size: Size::Long,
            offset:  offset,
            more:    Cursor::new(&EMPTY),
            src:     src,
        }
    }

    pub fn next(&mut self) -> io::Result<u16> {
        // Advance offset past virtually-read bytes
        let len = self.len();
        self.offset += len;

        // Advance buffer past virtually-read bytes
        let buf = match self.advance(len) {
            Ok (buf) => buf,
            Err(err) => { self.reset_to(&EMPTY); return Err(err) }
        };

        // Start new virtual read
        self.more = Cursor::new(buf);

        // Load next op word
        let op_word  = self.read_u16()?;
        self.op_word = op_word;
        self.op_size = Size::Long;
        Ok(op_word)
    }

    fn advance(&mut self, count: u64) -> io::Result<&'a [u8]> {
        use std::mem::transmute;
        // SAFETY: Transmute promotes buf from '_ to 'a.  This is OK because
        // buf's owner (self.src) has 'a, and buf is not modified outside of
        // this function.

        // Virtual reads become actual
        self.src.consume(count as usize);

        // Get more bytes for virtual reads
        let buf: &   [u8] = self.src.fill_buf()?;
        let buf: &'a [u8] = unsafe { transmute(buf) };

        Ok(buf)
    }

    fn reset(&mut self) {
        let buf = *self.more.get_ref();
        self.reset_to(buf);
    }

    fn reset_to(&mut self, buf: &'a [u8]) {
        self.op_word = 0;
        self.op_size = Size::Long;
        self.more    = Cursor::new(buf);
    }

    #[inline]
    pub fn op_word(&self) -> u16 {
        self.op_word
    }

    #[inline]
    pub fn op_size(&self) -> Size {
        self.op_size
    }

    #[inline]
    pub fn set_op_size(&mut self, size: Size) {
        self.op_size = size
    }

    #[inline]
    pub fn offset(&self) -> u64 {
        self.offset
    }

    #[inline]
    pub fn len(&self) -> u64 {
        self.more.position()
    }

    #[inline]
    pub fn read_i16(&mut self) -> io::Result<i16> {
        self.more.read_i16::<BE>()
            .map_err(|e| { self.reset(); e })
    }

    #[inline]
    pub fn read_u16(&mut self) -> io::Result<u16> {
        self.more.read_u16::<BE>()
            .map_err(|e| { self.reset(); e })
    }

    #[inline]
    pub fn read_u32(&mut self) -> io::Result<u32> {
        self.more.read_u32::<BE>()
            .map_err(|e| { self.reset(); e })
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor};
    use super::*;

    #[test]
    pub fn next() {
        let mut src = Cursor::new(vec![1, 2, 3, 4, 5]);
        let mut ctx = DecodeContext::new(&mut src, 0);

        assert_eq!( ctx.next().unwrap(), 0x0102 );
        assert_eq!( ctx.op_word(),       0x0102 );
        assert_eq!( ctx.next().unwrap(), 0x0304 );
        assert_eq!( ctx.op_word(),       0x0304 );
        assert_eq!( ctx.next().is_err(), true   );
    }

    #[test]
    pub fn next_read() {
        let mut src = Cursor::new(vec![1, 2, 3, 4, 5]);
        let mut ctx = DecodeContext::new(&mut src, 0);

        assert_eq!( ctx.next().unwrap(),     0x0102     );
        assert_eq!( ctx.op_word(),           0x0102     );
        assert_eq!( ctx.op_size(),           Size::Long );
        assert_eq!( ctx.offset(),            0          );
        assert_eq!( ctx.len(),               2          );

        assert_eq!( ctx.read_u16().unwrap(), 0x0304     );
        assert_eq!( ctx.op_word(),           0x0102     );
        assert_eq!( ctx.op_size(),           Size::Long );
        assert_eq!( ctx.offset(),            0          );
        assert_eq!( ctx.len(),               4          );

        assert_eq!( ctx.next().is_err(),     true       );
        assert_eq!( ctx.op_word(),           0          );
        assert_eq!( ctx.op_size(),           Size::Long );
        assert_eq!( ctx.offset(),            4          );
        assert_eq!( ctx.len(),               0          );
    }
}

