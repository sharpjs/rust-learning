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
use aex::fmt::{Code, Style};
use aex::ast::{Node, Prec, Precedence};
use num::BigInt;

/// An integer literal.
#[derive(Clone, Debug)]
pub struct Int<A = ()> {
    /// The value of the integer literal.
    pub value: BigInt,

    /// Annotation.
    pub ann: A,
}

impl Int {
    /// Creates a new `Int` with the given value and with `()` annotation.
    #[inline]
    pub fn new<V>(val: V) -> Self
    where V: Into<BigInt> {
        Self::new_with_ann(val, ())
    }
}

impl<A> Int<A> {
    /// Creates a new `Int` with the given value and annotation.
    #[inline]
    pub fn new_with_ann<V>(val: V, ann: A) -> Self
    where V: Into<BigInt>{
        Int { value: val.into(), ann: ann }
    }
}

impl<T> From<T> for Int where T: Into<BigInt> {
    /// Converts the given value to an `Int` with `()` annotation.
    #[inline]
    fn from(val: T) -> Self { Self::new(val) }
}

impl<A> Node for Int<A> {
    /// Annotation type.
    type Ann = A;

    /// Gets the annotation for this node.
    fn ann(&self) -> &A { &self.ann }
}

impl<A> Precedence for Int<A> {
    /// Gets the operator precedence level.
    #[inline(always)]
    fn precedence(&self) -> Prec { Prec::Atomic }
}

impl<A> Display for Int<A> {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "0x{:X}", &self.value)
    }
}

impl<A> Code for Int<A> {
    /// Formats the value as code, using the given formatter and style.
    #[inline]
    fn fmt<S: Style<A> + ?Sized>
          (&self, f: &mut Formatter, s: &S, p: Prec) -> fmt::Result {
        s.write_int(f, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aex::fmt::{Styled, IntelStyle};
    use aex::ast::Prec;
    use num::BigInt;

    #[test]
    fn new() {
        let i = Int::new(42);
        assert_eq!(i.value, BigInt::from(42));
        assert_eq!(i.ann, ());
    }

    #[test]
    fn new_with_ann() {
        let i = Int::new_with_ann(42, "a");
        assert_eq!(i.value, BigInt::from(42));
        assert_eq!(i.ann, "a");
    }

    #[test]
    fn from() {
        let i = Int::from(42);
        assert_eq!(i.value, BigInt::from(42));
        assert_eq!(i.ann, ());
    }

    #[test]
    fn precedence() {
        let i = Int::new(42);
        assert_eq!(i.precedence(), Prec::Atomic);
    }

    #[test]
    fn fmt() {
        let i = Int { value: BigInt::from(42), ann: "a" };
        let s = format!("{}", &i);
        assert_eq!(s, "0x2A");
    }

    #[test]
    fn fmt_asm() {
        let i = Int { value: BigInt::from(42), ann: "a" };
        let s = format!("{}", Styled::new(&i, &IntelStyle));
        assert_eq!(s, "42");
    }
}

