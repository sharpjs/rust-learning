// Built-In Types
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

use aex::pos::Source;
use aex::types::Type;
use aex::types::float::FloatSpec;
use aex::types::int::IntSpec;

struct BuiltInTypes<'a> {
    // Abstract
    pub int:      Type<'a>,
    pub float:    Type<'a>,
    // Concrete unsigned integer
    pub uint_8:   Type<'a>,
    pub uint_16:  Type<'a>,
    pub uint_32:  Type<'a>,
    pub uint_64:  Type<'a>,
    // Concrete signed integer
    pub int_8:    Type<'a>,
    pub int_16:   Type<'a>,
    pub int_32:   Type<'a>,
    pub int_64:   Type<'a>,
    // Concrete floating-point
    pub float_32: Type<'a>,
    pub float_64: Type<'a>,
}

macro_rules! int {
    () => (Type::Int(Source::BuiltIn, None));

    ($vw:expr, $sw:expr, $sg:expr) => (Type::Int(
        Source::BuiltIn,
        Some(IntSpec { value_width: $vw, store_width: $sw, signed: $sg })
    ))
}

macro_rules! float {
    () => (Type::Float(Source::BuiltIn, None));

    ($vw:expr, $sw:expr) => (Type::Float(
        Source::BuiltIn,
        Some(FloatSpec { value_width: $vw, store_width: $sw })
    ))
}

impl<'a> Default for BuiltInTypes<'a> {
    fn default() -> Self {
        BuiltInTypes {
            // Abstract
            int:      int!   (),
            float:    float! (),
            // Concrete unsigned integer
            uint_8:   int!   ( 8,  8, false),
            uint_16:  int!   (16, 16, false),
            uint_32:  int!   (32, 32, false),
            uint_64:  int!   (64, 64, false),
            // Concrete signed integer
            int_8:    int!   ( 8,  8, true),
            int_16:   int!   (16, 16, true),
            int_32:   int!   (32, 32, true),
            int_64:   int!   (64, 64, true),
            // Concrete floating-point
            float_32: float! (32, 32),
            float_64: float! (64, 64),
        }
    }
}

