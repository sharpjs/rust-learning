// AST: Integer Literals
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

use std::fmt::{self, Display, Formatter};
use aex::asm::{AsmDisplay, AsmStyle};
use num::BigInt;

/// An integer literal.
#[derive(Clone, Debug)]
pub struct Int<C = ()> {
    /// The value of the integer literal.
    pub value: BigInt,

    /// A context value.
    pub context: C,
}

impl Int {
    /// Creates a new `Int` with the given value and with `()` context.
    #[inline]
    pub fn new(val: BigInt) -> Self {
        Int { value: val, context: () }
    }
}

impl<C> Int<C> {
    /// Creates a new `Int` with the given value and context.
    #[inline]
    pub fn new_with_context(val: BigInt, ctx: C) -> Self {
        Int { value: val, context: ctx }
    }
}

impl<T> From<T> for Int where BigInt: From<T> {
    /// Converts the given value to an `Int` with `()` context.
    #[inline]
    fn from(v: T) -> Self {
        Int::new(BigInt::from(v))
    }
}

impl<C> Display for Int<C> {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "0x{:X}", &self.value)
    }
}

impl<C> AsmDisplay<C> for Int<C> {
    /// Formats the value as assembly code, using the given formatter and
    /// assembly style.
    #[inline]
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle<C>) -> fmt::Result {
        s.write_int(f, self)
    }
}

#[cfg(test)]
mod tests {
    use aex::asm::{Asm, IntelStyle};
    use num::BigInt;
    use super::*;

    #[test]
    fn new() {
        let i = Int::new(BigInt::from(42));
        assert_eq!(i.value, BigInt::from(42));
        assert_eq!(i.context, ());
    }

    #[test]
    fn new_with_context() {
        let i = Int::new_with_context(BigInt::from(42), "a");
        assert_eq!(i.value, BigInt::from(42));
        assert_eq!(i.context, "a");
    }

    #[test]
    fn from() {
        let i = Int::from(42);
        assert_eq!(i.value, BigInt::from(42));
        assert_eq!(i.context, ());
    }

    #[test]
    fn fmt() {
        let i = Int { value: BigInt::from(42), context: "a" };
        let s = format!("{}", &i);
        assert_eq!(s, "0x2A");
    }

    #[test]
    fn fmt_asm() {
        let i = Int { value: BigInt::from(42), context: "a" };
        let s = format!("{}", Asm(&i, &IntelStyle));
        assert_eq!(s, "42");
    }
}

