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

// Abstract integer
pub static INT:   Type<'static> = int!   ();

// Concrete unsigned integer
pub static U8:    Type<'static> = int!   ( 8,  8, false);
pub static U16:   Type<'static> = int!   (16, 16, false);
pub static U32:   Type<'static> = int!   (32, 32, false);
pub static U64:   Type<'static> = int!   (64, 64, false);

// Concrete signed integer
pub static I8:    Type<'static> = int!   ( 8,  8, true);
pub static I16:   Type<'static> = int!   (16, 16, true);
pub static I32:   Type<'static> = int!   (32, 32, true);
pub static I64:   Type<'static> = int!   (64, 64, true);

// Abstract floating-point
pub static FLOAT: Type<'static> = float! ();

// Concrete floating-point
pub static F32:   Type<'static> = float! (32, 32);
pub static F64:   Type<'static> = float! (64, 64);

