// Reader for Decoding
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

use std::io::Result;
use std::mem::size_of;

use aex::util::{ByteOrder, Endian};
use super::RewindRead;

macro_rules! read_as {
    ($r:expr, $t:ty) => {{
        let order = $r.byte_order();
        let bytes = $r.read_bytes(size_of::<$t>())?;
        let value = unsafe { *( bytes.as_ptr() as *const $t ) };
        Ok(value.to_order(order))
    }};
}

/// A reader suitable for decoding machine code.
///
/// A `DecodeRead` views a stream as consumed, pending, and unread bytes:
///
/// ```text
/// consumed   pending       unread
/// ...........**************????????>
///            |<----------->|
///            |      |
///            |      |_________ pending_len
///            |________________ lma + reloc = vma
/// ```
///
/// A `DecodeRead` maintains a current byte order, which is used when reading
/// multi-byte primitive values (`u32`, etc.).
///
pub trait DecodeRead {
    /// Returns the load address (LMA) of the reader, considering only consumed bytes.
    fn lma(&self) -> u64;

    /// Returns the relocated address (VMA) of the reader, considering only consumed bytes.
    #[inline]
    fn vma(&self) -> u64 {
        self.lma().wrapping_add(self.reloc())
    }

    /// Returns the relocation amount (VMA - LMA) of the reader.
    fn reloc(&self) -> u64;

    /// Sets the relocation amount (VMA - LMA) of the reader.
    fn set_reloc(&mut self, reloc: u64);

    /// Returns the byte order used to decode multi-byte values.
    fn byte_order(&self) -> ByteOrder;

    /// Sets the byte order used to decode multi-byte values.
    fn set_byte_order(&mut self, order: ByteOrder);

    /// Returns the count of pending bytes of the reader.
    fn pending_len(&self) -> usize;

    /// Returns the pending bytes of the reader as a slice.
    fn pending_bytes(&self) -> &[u8];

    /// Consumes pending bytes and advances the rewind point to the next unread byte.
    fn consume(&mut self);

    /// Rewinds pending bytes, making them readable again.
    fn rewind(&mut self);

    /// Reads `n` bytes, returning them as a slice.  The bytes become pending.
    fn read_bytes(&mut self, n: usize) -> Result<&[u8]>;

    /// Reads a `u8`.  The byte becomes pending.
    #[inline]
    fn read_u8(&mut self) -> Result<u8> {
        Ok(self.read_bytes(1)?[0])
    }

    /// Reads an `i8`.  The byte becomes pending.
    #[inline]
    fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    /// Reads a `u16`.  The bytes become pending.
    #[inline]
    fn read_u16(&mut self) -> Result<u16> {
        read_as!(self, u16)
    }

    /// Reads an `i16`.  The bytes become pending.
    #[inline]
    fn read_i16(&mut self) -> Result<i16> {
        read_as!(self, i16)
    }

    /// Reads a `u32`.  The bytes become pending.
    #[inline]
    fn read_u32(&mut self) -> Result<u32> {
        read_as!(self, u32)
    }

    /// Reads an `i32`.  The bytes become pending.
    #[inline]
    fn read_i32(&mut self) -> Result<i32> {
        read_as!(self, i32)
    }

    /// Reads a `u64`.  The bytes become pending.
    #[inline]
    fn read_u64(&mut self) -> Result<u64> {
        read_as!(self, u64)
    }

    /// Reads an `i64`.  The bytes become pending.
    #[inline]
    fn read_i64(&mut self) -> Result<i64> {
        read_as!(self, i64)
    }
}

#[derive(Clone, Debug)]
pub struct DecodeReader<R: RewindRead> {
    inner: R,
    order: ByteOrder,
    reloc: u64,
}

impl<R: RewindRead> DecodeReader<R> {
    pub fn new(inner: R, order: ByteOrder, reloc: u64) -> Self {
        Self { inner, order, reloc }
    }
}

impl<R: RewindRead> DecodeRead for DecodeReader<R> {
    /// Returns the load address (LMA) of the reader, considering only consumed bytes.
    #[inline]
    fn lma(&self) -> u64 {
        self.inner.consumed_pos() as u64
    }

    /// Returns the relocation amount (VMA - LMA) of the reader.
    #[inline]
    fn reloc(&self) -> u64 {
        self.reloc
    }

    /// Sets the relocation amount (VMA - LMA) of the reader.
    #[inline]
    fn set_reloc(&mut self, reloc: u64) {
        self.reloc = reloc;
    }

    /// Returns the byte order used to decode multi-byte values.
    #[inline]
    fn byte_order(&self) -> ByteOrder {
        self.order
    }

    /// Sets the byte order used to decode multi-byte values.
    #[inline]
    fn set_byte_order(&mut self, order: ByteOrder) {
        self.order = order;
    }

    /// Returns the count of pending bytes of the reader.
    #[inline]
    fn pending_len(&self) -> usize {
        self.inner.pending_len()
    }

    /// Returns the pending bytes of the reader as a slice.
    #[inline]
    fn pending_bytes(&self) -> &[u8] {
        self.inner.pending_bytes()
    }

    /// Consumes pending bytes and advances the rewind point to the next unread byte.
    #[inline]
    fn consume(&mut self) {
        self.inner.consume()
    }

    /// Rewinds pending bytes, making them readable again.
    #[inline]
    fn rewind(&mut self) {
        self.inner.rewind()
    }

    /// Reads `n` bytes, returning them as a slice.  The bytes become pending.
    #[inline]
    fn read_bytes(&mut self, n: usize) -> Result<&[u8]> {
        self.inner.read_bytes(n)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use aex::util::{BE, LE};
    use super::super::RewindReader;
    use super::*;

    #[test]
    fn lma() {
        let mut r = reader();
        assert_eq!(r.lma(), 0, "initial");
        r.read_bytes(2).unwrap();
        assert_eq!(r.lma(), 0, "after read");
        r.consume();
        assert_eq!(r.lma(), 2, "after consume");
    }

    #[test]
    fn vma() {
        let mut r = reader();
        assert_eq!(r.vma(), 0x100, "initial");
        r.read_bytes(2).unwrap();
        assert_eq!(r.vma(), 0x100, "after read");
        r.consume();
        assert_eq!(r.vma(), 0x102, "after consume");
        r.set_reloc(0x400);
        assert_eq!(r.vma(), 0x402, "after relocation change");
    }

    #[test]
    fn reloc() {
        let mut r = reader();
        assert_eq!(r.reloc(), 0x100);
        r.set_reloc(0x400);
        assert_eq!(r.reloc(), 0x400);
    }

    #[test]
    fn byte_order() {
        let mut r = reader();
        assert_eq!(r.byte_order(), BE);
        r.set_byte_order(LE);
        assert_eq!(r.byte_order(), LE);
    }

    #[test]
    fn read_bytes() {
        let mut r = reader();
        assert_eq!(r.read_bytes(2).unwrap(), &[0x12, 0x34]);

        // read_bytes() should be byte-order agnostic
        let mut r = reader();
        r.set_byte_order(LE);
        assert_eq!(r.read_bytes(2).unwrap(), &[0x12, 0x34]);
    }

    #[test]
    fn read_u8() {
        let mut r = reader();
        assert_eq!(r.read_u8().unwrap(), 0x12);

        // read_u8() should be byte-order agnostic
        let mut r = reader();
        r.set_byte_order(LE);
        assert_eq!(r.read_u8().unwrap(), 0x12);
    }

    #[test]
    fn read_i8() {
        let mut r = reader();
        assert_eq!(r.read_i8().unwrap(), 0x12);

        // read_i8() should be byte-order agnostic
        let mut r = reader();
        r.set_byte_order(LE);
        assert_eq!(r.read_i8().unwrap(), 0x12);
    }

    #[test]
    fn read_u16_be() {
        let mut r = reader();
        assert_eq!(r.read_u16().unwrap(), 0x1234);
    }

    #[test]
    fn read_u16_le() {
        let mut r = reader();
        r.set_byte_order(LE);
        assert_eq!(r.read_u16().unwrap(), 0x3412);
    }

    #[test]
    fn read_i16_be() {
        let mut r = reader();
        assert_eq!(r.read_i16().unwrap(), 0x1234);
    }

    #[test]
    fn read_i16_le() {
        let mut r = reader();
        r.set_byte_order(LE);
        assert_eq!(r.read_i16().unwrap(), 0x3412);
    }

    #[test]
    fn read_u32_be() {
        let mut r = reader();
        assert_eq!(r.read_u32().unwrap(), 0x12345678);
    }

    #[test]
    fn read_u32_le() {
        let mut r = reader();
        r.set_byte_order(LE);
        assert_eq!(r.read_u32().unwrap(), 0x78563412);
    }

    #[test]
    fn read_i32_be() {
        let mut r = reader();
        assert_eq!(r.read_i32().unwrap(), 0x12345678);
    }

    #[test]
    fn read_i32_le() {
        let mut r = reader();
        r.set_byte_order(LE);
        assert_eq!(r.read_i32().unwrap(), 0x78563412);
    }

    #[test]
    fn read_u64_be() {
        let mut r = reader();
        assert_eq!(r.read_u64().unwrap(), 0x123456789ABCDEF0);
    }

    #[test]
    fn read_u64_le() {
        let mut r = reader();
        r.set_byte_order(LE);
        assert_eq!(r.read_u64().unwrap(), 0xF0DEBC9A78563412);
    }

    #[test]
    fn read_i64_be() {
        let mut r = reader();
        assert_eq!(r.read_i64().unwrap(), 0x123456789ABCDEF0);
    }

    #[test]
    fn read_i64_le() {
        let mut r = reader();
        r.set_byte_order(LE);
        assert_eq!(r.read_i64().unwrap(), -0x0F21436587A9CBEE);
    }

    fn reader() -> Box<DecodeRead> {
        let src = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let src = Cursor::new(src);
        let src = RewindReader::new(src);
        Box::new(DecodeReader::new(src, BE, 0x100))
    }
}

