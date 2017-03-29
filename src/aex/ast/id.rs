// AST: Identifiers
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

/// An identifier.
#[derive(Clone, Copy, Debug)]
pub struct Id<'a, A = ()> {
    /// The name of the identifier.
    pub name: &'a str,

    /// Annotation.
    pub ann: A,
}

impl<'a> Id<'a> {
    /// Creates a new `Id` with the given name and with `()` annotation.
    #[inline]
    pub fn new(name: &'a str) -> Self {
        Self::new_with_ann(name, ())
    }
}

impl<'a, A> Id<'a, A> {
    /// Creates a new `Id` with the given name and annotation.
    #[inline]
    pub fn new_with_ann(name: &'a str, ann: A) -> Self {
        Id { name: name, ann: ann }
    }
}

impl<'a> From<&'a str> for Id<'a> {
    /// Converts the given value to an `Id` with `()` annotation.
    #[inline]
    fn from(name: &'a str) -> Self { Self::new(name) }
}

impl<'a, A> Node for Id<'a, A> {
    /// Annotation type.
    type Ann = A;

    /// Gets the annotation for this node.
    fn ann(&self) -> &A { &self.ann }
}

impl<'a, A> Precedence for Id<'a, A> {
    /// Gets the operator precedence level.
    #[inline(always)]
    fn precedence(&self) -> Prec { Prec::Atomic }
}

impl<'a, A> Display for Id<'a, A> {
    /// Formats the value using the given formatter.
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.name)
    }
}

impl<'a, A> Code for Id<'a, A> {
    /// Formats the value as code, using the given formatter and style.
    #[inline]
    fn fmt<S: Style<A> + ?Sized>
          (&self, f: &mut Formatter, s: &S) -> fmt::Result {
        s.write_id(f, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aex::fmt::{ToStyled, IntelStyle};
    use aex::ast::Prec;

    #[test]
    fn new() {
        let i = Id::new("a");
        assert_eq!(i.name, "a");
        assert_eq!(i.ann, ());
    }

    #[test]
    fn new_with_ann() {
        let i = Id::new_with_ann("a", 42);
        assert_eq!(i.name, "a");
        assert_eq!(i.ann, 42);
    }

    #[test]
    fn from() {
        let i = Id::from("a");
        assert_eq!(i.name, "a");
        assert_eq!(i.ann, ());
    }

    #[test]
    fn precedence() {
        let i = Id::new("a");
        assert_eq!(i.precedence(), Prec::Atomic);
    }

    #[test]
    fn fmt() {
        let i = Id { name: "a", ann: 42 };
        let s = format!("{}", &i);
        assert_eq!(s, "a");
    }

    #[test]
    fn fmt_asm() {
        let i = Id { name: "a", ann: 42 };
        let s = format!("{}", i.styled(&IntelStyle));
        assert_eq!(s, "a");
    }
}

