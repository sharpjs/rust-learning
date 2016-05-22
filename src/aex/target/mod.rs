// Target Architectures
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

mod cf; // Freescale ColdFire

pub use self::cf::ColdFire;

use aex::types::form::TypeInfo;

pub trait Target: Types {}

pub trait Types {
    fn is_valid_ptr(&TypeInfo) -> bool;
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use aex::types::form::TypeInfo;

    pub struct TestTarget;

    impl Target for TestTarget {}

    impl Types for TestTarget {
        fn is_valid_ptr(t: &TypeInfo) -> bool { true }
    }
}

