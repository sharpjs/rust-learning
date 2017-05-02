// Byte Order Conversion
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

use super::DecodeRead;

macro_rules! read_int {
    ($r:expr, $t:ty, $f:ident) => {{
        let n = size_of::<$t>();
        let p = $r.read_exact(n)?.as_ptr() as *const $t;
        let v = unsafe { *p };
        Ok(v.$f())
    }};
}

pub trait ByteOrderRead: DecodeRead {
    #[inline]
    fn read_u8(&mut self) -> Result<u8> {
        unsafe { Ok(*self.read_exact(1)?.get_unchecked(0)) }
    }

    #[inline]
    fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    #[inline]
    fn read_u16_be(&mut self) -> Result<u16> {
        read_int!(self, u16, to_be)
    }

    #[inline]
    fn read_u32_be(&mut self) -> Result<u32> {
        read_int!(self, u32, to_be)
    }

    #[inline]
    fn read_u64_be(&mut self) -> Result<u64> {
        read_int!(self, u64, to_be)
    }

    #[inline]
    fn read_i16_be(&mut self) -> Result<i16> {
        Ok(self.read_u16_be()? as i16)
    }

    #[inline]
    fn read_i32_be(&mut self) -> Result<i32> {
        Ok(self.read_u32_be()? as i32)
    }

    #[inline]
    fn read_i64_be(&mut self) -> Result<i64> {
        Ok(self.read_u64_be()? as i64)
    }

    #[inline]
    fn read_u16_le(&mut self) -> Result<u16> {
        read_int!(self, u16, to_le)
    }

    #[inline]
    fn read_u32_le(&mut self) -> Result<u32> {
        read_int!(self, u32, to_le)
    }

    #[inline]
    fn read_u64_le(&mut self) -> Result<u64> {
        read_int!(self, u64, to_le)
    }

    #[inline]
    fn read_i16_le(&mut self) -> Result<i16> {
        Ok(self.read_u16_le()? as i16)
    }

    #[inline]
    fn read_i32_le(&mut self) -> Result<i32> {
        Ok(self.read_u32_le()? as i32)
    }

    #[inline]
    fn read_i64_le(&mut self) -> Result<i64> {
        Ok(self.read_u64_le()? as i64)
    }
}


#[cfg(test)]
mod tests {
/*
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
*/
}

