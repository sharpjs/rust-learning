// Built-In Types
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

use aex::pos::Source;
use aex::types::{Type, IntTy, FloatTy};
use aex::types::float::FloatSpec;
use aex::types::int::IntSpec;

macro_rules! int {
    () => (Type::Int(IntTy::Abstract));

    ($vw:expr, $sw:expr, $sg:expr) => (Type::Int(
        IntTy::Concrete {
            spec: IntSpec {
                value_width: $vw,
                store_width: $sw,
                signed:      $sg,
            },
            src: Source::BuiltIn,
        }
    ))
}

macro_rules! float {
    () => (Type::Float(FloatTy::Abstract));

    ($vw:expr, $sw:expr) => (Type::Float(
        FloatTy::Concrete {
            spec: FloatSpec {
                value_width: $vw,
                store_width: $sw
            },
            src: Source::BuiltIn,
        }
    ))
}

macro_rules! types {
    ($($id:ident = $ty:expr;)*) => ($(
        pub static $id: Type<'static> = $ty;
    )*)
}

types! {
    // Abstract integer
    INT   = int!();

    // Concrete unsigned integer
    U8    = int!( 8,  8, false);
    U16   = int!(16, 16, false);
    U32   = int!(32, 32, false);
    U64   = int!(64, 64, false);

    // Concrete signed integer
    I8    = int!( 8,  8, true);
    I16   = int!(16, 16, true);
    I32   = int!(32, 32, true);
    I64   = int!(64, 64, true);

    // Abstract floating-point
    FLOAT = float!();

    // Concrete floating-point
    F32   = float!(32, 32);
    F64   = float!(64, 64);
}

