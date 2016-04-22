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

macro_rules! types {
    ($($name:ident: $ty:expr;)*) => {$(
        pub static $name: Type<'static> = $ty;
    )*};
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

types! {
    // Abstract
    INT:    int!    ();
    FLOAT:  float!  ();

    // Concrete unsigned integer
     U8:    int!    ( 8,  8, false);
    U16:    int!    (16, 16, false);
    U32:    int!    (32, 32, false);
    U64:    int!    (64, 64, false);

    // Concrete signed integer
     I8:    int!    ( 8,  8, true);
    I16:    int!    (16, 16, true);
    I32:    int!    (32, 32, true);
    I64:    int!    (64, 64, true);

    // Concrete floating-point
    F32:    float!  (32, 32);
    F64:    float!  (64, 64);
}

