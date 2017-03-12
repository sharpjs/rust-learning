// Identifiers
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

/// An identifier.
#[derive(Clone, Copy, Debug)]
pub struct Id<'a, C = ()> {
    /// The name of the identifier.
    pub name: &'a str,

    /// A context value.
    pub context: C,
}

impl<'a> Id<'a, ()> {
    /// Creates a new `Id` with the given name and with `()` context.
    #[inline]
    pub fn new(name: &'a str) -> Self {
        Id { name: name, context: () }
    }
}

impl<'a, C> Id<'a, C> {
    /// Creates a new `Id` with the given name and context.
    #[inline]
    pub fn new_with_context(name: &'a str, ctx: C) -> Self {
        Id { name: name, context: ctx }
    }
}

impl<'a, C> Display for Id<'a, C> {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.name)
    }
}

impl<'a, C> AsmDisplay<C> for Id<'a, C> {
    #[inline]
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle<C>) -> fmt::Result {
        s.write_id(f, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aex::asm::{Asm, IntelStyle};

    #[test]
    pub fn new() {
        let id = Id::new("a");
        assert_eq!(id.name, "a");
        assert_eq!(id.context, ());
    }

    #[test]
    pub fn new_with_context() {
        let id = Id::new_with_context("a", 42);
        assert_eq!(id.name, "a");
        assert_eq!(id.context, 42);
    }

    #[test]
    pub fn fmt() {
        let id = Id { name: "a", context: 42 };
        let s = format!("{}", &id);
        assert_eq!(s, "a");
    }

    #[test]
    pub fn fmt_asm() {
        let id = Id { name: "a", context: 42 };
        let s = format!("{}", Asm(&id, &IntelStyle));
        assert_eq!(s, "a");
    }
}

