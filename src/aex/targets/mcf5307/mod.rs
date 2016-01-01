// MCF5307 Target
//
// This file is part of AEx.
// Copyright (C) 2015 Jeffrey Sharp
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

mod loc;
mod code_gen;

use aex::types::*;
use aex::targets::*;

pub struct Mcf5307 {
    ptr_type: Type<'static>
}

impl Mcf5307 {
    fn new() -> Self {
        Mcf5307 {
            ptr_type: Type::Ptr(
                Box::new(Type::Ref("int")),
                Box::new(Type::Ref("u8" ))
            )
        }
    }
}

impl Default for Mcf5307 {
    #[inline]
    fn default() -> Self { Self::new() }
}

impl Target for Mcf5307 {
    #[inline]
    fn label_type(&self) -> &Type<'static> { &self.ptr_type }
}

