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
use aex::asm::{AsmDisplay, AsmStyle};
use aex::ast::{Context, Prec, Precedence};

/// An identifier.
#[derive(Clone, Copy, Debug)]
pub struct Id<'a, C = ()> {
    /// The name of the identifier.
    pub name: &'a str,

    /// A context value.
    pub context: C,
}

impl<'a> Id<'a> {
    /// Creates a new `Id` with the given name and with `()` context.
    #[inline]
    pub fn new(name: &'a str) -> Self {
        Self::new_with_context(name, ())
    }
}

impl<'a, C> Id<'a, C> {
    /// Creates a new `Id` with the given name and context.
    #[inline]
    pub fn new_with_context(name: &'a str, ctx: C) -> Self {
        Id { name: name, context: ctx }
    }
}

impl<'a> From<&'a str> for Id<'a> {
    /// Converts the given value to an `Id` with `()` context.
    #[inline]
    fn from(name: &'a str) -> Self { Self::new(name) }
}

impl<'a, C> Context for Id<'a, C> {
    /// Type of the context value.
    type Context = C;

    /// Gets the context value.
    fn context(&self) -> &C { &self.context }
}

impl<'a, C> Precedence for Id<'a, C> {
    /// Gets the operator precedence level.
    #[inline(always)]
    fn precedence(&self) -> Prec { Prec::Atomic }
}

impl<'a, C> Display for Id<'a, C> {
    /// Formats the value using the given formatter.
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.name)
    }
}

impl<'a, C> AsmDisplay<C> for Id<'a, C> {
    /// Formats the value as assembly code, using the given formatter and
    /// assembly style.
    #[inline]
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle<C>, p: Prec) -> fmt::Result {
        s.write_id(f, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aex::asm::{Asm, IntelStyle};
    use aex::ast::Prec;

    #[test]
    fn new() {
        let i = Id::new("a");
        assert_eq!(i.name, "a");
        assert_eq!(i.context, ());
    }

    #[test]
    fn new_with_context() {
        let i = Id::new_with_context("a", 42);
        assert_eq!(i.name, "a");
        assert_eq!(i.context, 42);
    }

    #[test]
    fn from() {
        let i = Id::from("a");
        assert_eq!(i.name, "a");
        assert_eq!(i.context, ());
    }

    #[test]
    fn precedence() {
        let i = Id::new("a");
        assert_eq!(i.precedence(), Prec::Atomic);
    }

    #[test]
    fn fmt() {
        let i = Id { name: "a", context: 42 };
        let s = format!("{}", &i);
        assert_eq!(s, "a");
    }

    #[test]
    fn fmt_asm() {
        let i = Id { name: "a", context: 42 };
        let s = format!("{}", Asm::new(&i, &IntelStyle));
        assert_eq!(s, "a");
    }
}

