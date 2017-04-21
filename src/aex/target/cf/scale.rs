// ColdFire Indexing Scales
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

use self::Scale::*;

/// ColdFire indexed addressing scales.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Scale {
    /// Byte
    Byte,

    /// Word (2 bytes)
    Word,

    /// Longword (4 bytes)
    Long,

    /// Quadword (8 bytes) (not supported by all models)
    Quad
}

impl Scale {
    /// Returns the scale with the given size in bytes.
    pub fn with_size(size: u8) -> Option<Self> {
        match size {
            1 => Some(Byte),
            2 => Some(Word),
            4 => Some(Long),
            8 => Some(Quad),
            _ => None,
        }
    }

    /// Returns the scale's size in bytes.
    #[inline]
    pub fn size(self) -> u8 {
        match self {
            Byte => 1,
            Word => 2,
            Long => 4,
            Quad => 8,
        }
    }

    /// Decodes a scale from the given instruction bits.
    pub fn decode(word: u16, pos: u8) -> Self {
        let scale = word >> pos & 0b11;
        match scale {
            0 => Byte,
            1 => Word,
            2 => Long,
            3 => Quad,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_size() {
        assert_eq!(Scale::with_size(1), Some(Byte));
        assert_eq!(Scale::with_size(2), Some(Word));
        assert_eq!(Scale::with_size(4), Some(Long));
        assert_eq!(Scale::with_size(8), Some(Quad));
        assert_eq!(Scale::with_size(0), None);
    }

    #[test]
    fn size() {
        assert_eq!(Byte.size(), 1);
        assert_eq!(Word.size(), 2);
        assert_eq!(Long.size(), 4);
        assert_eq!(Quad.size(), 8);
    }

    #[test]
    fn decode() {
        assert_eq!(Scale::decode(0b0000, 2), Byte);
        assert_eq!(Scale::decode(0b0100, 2), Word);
        assert_eq!(Scale::decode(0b1000, 2), Long);
        assert_eq!(Scale::decode(0b1100, 2), Quad);
    }
}

