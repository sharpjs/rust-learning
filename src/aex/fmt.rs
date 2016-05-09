// Formatting Trait
//
// This file is part of AEx.
// Copyright (C) 2016 Jeffrey Sharp
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

use std::fmt::Display;
use std::io::{self, Write};

pub trait Format<C> {
    fn fmt<W: Write>(&self, &C, &mut W) -> io::Result<()>;
}

impl<C, T: Display> Format<C> for T {
    fn fmt<W: Write>(&self, _: &C, w: &mut W) -> io::Result<()> {
        write!(w, "{}", self)
    }
}

#[macro_export]
macro_rules! format_with {
    ($obj:expr, $ctx:expr) => {{
        use std::io::Cursor;
        use aex::fmt::Format;

        let     vec = Vec::with_capacity(16);
        let mut buf = Cursor::new(vec);

        $obj.fmt($ctx, &mut buf).unwrap();

        let vec = buf.into_inner();
        String::from_utf8(vec).unwrap()
    }}
}

