// AST: Registers
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
use aex::ast::{Prec, Precedence};

/// A register.
#[derive(Clone, Copy, Debug)]
pub struct Reg<'a, C = ()> {
    /// The name of the register.
    pub name: &'a str,

    /// A context value.
    pub context: C,
}

impl<'a> Reg<'a> {
    /// Creates a new `Reg` with the given name and with `()` context.
    #[inline]
    pub fn new(name: &'a str) -> Self {
        Self::new_with_context(name, ())
    }
}

impl<'a, C> Reg<'a, C> {
    /// Creates a new `Reg` with the given name and context.
    #[inline]
    pub fn new_with_context(name: &'a str, ctx: C) -> Self {
        Reg { name: name, context: ctx }
    }
}

impl<'a> From<&'a str> for Reg<'a> {
    /// Converts the given value to an `Reg` with `()` context.
    #[inline]
    fn from(name: &'a str) -> Self { Self::new(name) }
}

impl<'a, C> Precedence for Reg<'a, C> {
    /// Gets the operator precedence level.
    #[inline(always)]
    fn precedence(&self) -> Prec { Prec::Atomic }
}

impl<'a, C> Display for Reg<'a, C> {
    /// Formats the value using the given formatter.
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.name)
    }
}

impl<'a, C> AsmDisplay<C> for Reg<'a, C> {
    /// Formats the value as assembly code, using the given formatter and
    /// assembly style.
    #[inline]
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle<C>, p: Prec) -> fmt::Result {
        s.write_reg(f, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aex::asm::{Asm, IntelStyle};
    use aex::ast::Prec;

    #[test]
    fn new() {
        let r = Reg::new("a");
        assert_eq!(r.name, "a");
        assert_eq!(r.context, ());
    }

    #[test]
    fn new_with_context() {
        let r = Reg::new_with_context("a", 42);
        assert_eq!(r.name, "a");
        assert_eq!(r.context, 42);
    }

    #[test]
    fn from() {
        let r = Reg::from("a");
        assert_eq!(r.name, "a");
        assert_eq!(r.context, ());
    }

    #[test]
    fn precedence() {
        let i = Reg::new("a");
        assert_eq!(i.precedence(), Prec::Atomic);
    }

    #[test]
    fn fmt() {
        let r = Reg { name: "a", context: 42 };
        let s = format!("{}", &r);
        assert_eq!(s, "a");
    }

    #[test]
    fn fmt_asm() {
        let r = Reg { name: "a", context: 42 };
        let s = format!("{}", Asm(&r, &IntelStyle));
        assert_eq!(s, "a");
    }
}

