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
use aex::types::res::{TypeRes, TypeForm};

macro_rules! int {
    ($vw:expr, $sw:expr, $sg:expr) => (TypeForm::Inty(
        Some(IntSpec { value_width: $vw, store_width: $sw, signed: $sg })
    ));
}

macro_rules! float {
    ($vw:expr, $sw:expr) => (TypeForm::Floaty(
        Some(FloatSpec { value_width: $vw, store_width: $sw })
    ));
}

macro_rules! types {
    ($($name:ident: $id:expr, $form:expr;)*) => {$(
        pub static $name: TypeRes<'static> = TypeRes {
            def: &Type::Ident(Source::BuiltIn, $id), form: $form
        };
    )*};
}

types! {
    // Abstract
    INT:    "(int)",     TypeForm::Inty   (None);
    FLOAT:  "(float)",   TypeForm::Floaty (None);
    OPAQUE: "(unknown)", TypeForm::Opaque;

    // Concrete unsigned integer
     U8:  "u8", int!( 8,  8, false);
    U16: "u16", int!(16, 16, false);
    U32: "u32", int!(32, 32, false);
    U64: "u64", int!(64, 64, false);

    // Concrete signed integer
     I8:  "i8", int!( 8,  8, true);
    I16: "i16", int!(16, 16, true);
    I32: "i32", int!(32, 32, true);
    I64: "i64", int!(64, 64, true);

    // Concrete floating-point
    F32: "f32", float!(32, 32);
    F64: "f64", float!(64, 64);
}

