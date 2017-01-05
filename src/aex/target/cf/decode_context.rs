// ColdFire Decode Context
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

#[derive(Clone, Debug)]
pub struct DecodeContext<'a> {
    pub word: u16,              // operation word
        ext:  ExtensionWord,    // operation word
    pub more: Cursor<&'a [u8]>, // extension word source
  //pub size: u8,               // size suffix
}

#[derive(Clone, Debug)]
enum ExtensionWord {
    None,
    Word([u8; 2]),
    Long([u8; 4]),
}

impl<'a> DecodeContext<'a> {
    pub fn load<R: BufRead>(src: &'a mut R) -> io::Result<Self> {
        let mut more = Cursor::new(src.fill_buf()?);
        let     word = more.read_u16::<BE>()?;
        Ok(DecodeContext {
            word: word,
            ext:  ExtensionWord::None,
            more: more
        })
    }

    pub fn position(&self) -> usize {
        self.more.position() as usize
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, Cursor};
    use super::*;

    #[test]
    pub fn load() {
        let mut src = Cursor::new(vec![0, 1, 2, 3, 4, 5, 6]);

        loop {
            let p = {
                let r = match DecodeContext::load(&mut src) {
                    Ok(r) => r, Err(_) => break
                };
                r.position()
            };
            src.consume(p)
        }
    }
}

