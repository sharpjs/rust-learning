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

use std::fmt::{self, Formatter};
use std::io;

use aex::fmt::{Code, Style};
use aex::util::invalid;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum Scale {
    Byte = 1,
    Word = 2,
    Long = 4,
}

impl Scale {
    pub fn with_size(size: u8) -> Option<Self> {
        match size {
            1 => Some(Scale::Byte),
            2 => Some(Scale::Word),
            4 => Some(Scale::Long),
            _ => None,
        }
    }

    #[inline]
    pub fn size(self) -> u8 {
        self as u8
    }

    pub fn decode(word: u16, pos: u8) -> io::Result<Self> {
        let scale = word >> pos & 0b11;
        match scale {
            0 => Ok(Scale::Byte),
            1 => Ok(Scale::Word),
            2 => Ok(Scale::Long),
            _ => invalid(),
        }
    }
}

impl Code for Scale {
    fn fmt(&self, f: &mut Formatter, s: &Style) -> fmt::Result {
        s.write_scale(f, self.size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_size() {
        assert_eq!(Scale::with_size(1), Some(Scale::Byte));
        assert_eq!(Scale::with_size(2), Some(Scale::Word));
        assert_eq!(Scale::with_size(4), Some(Scale::Long));
        assert_eq!(Scale::with_size(0), None);
    }

    #[test]
    fn size() {
        assert_eq!(Scale::Byte.size(), 1);
        assert_eq!(Scale::Word.size(), 2);
        assert_eq!(Scale::Long.size(), 4);
    }

    #[test]
    fn decode() {
        assert_eq!(Scale::decode(0b0000, 2).unwrap(), Scale::Byte);
        assert_eq!(Scale::decode(0b0100, 2).unwrap(), Scale::Word);
        assert_eq!(Scale::decode(0b1000, 2).unwrap(), Scale::Long);
        assert_eq!(Scale::decode(0b1100, 2).is_err(), true);
    }
}

