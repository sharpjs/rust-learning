// Abstract Syntax Tree
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

// -----------------------------------------------------------------------------

/// An identifier.
#[derive(Clone, Copy, Debug)]
pub struct Id<'a, C = ()> {
    /// The name of the identifier.
    pub name: &'a str,

    /// A context value.
    pub ctx: C,
}

impl<'a> Id<'a, ()> {
    /// Creates a new `Id` with the given name and with `()` context.
    #[inline]
    pub fn new(name: &'a str) -> Self {
        Id { name: name, ctx: () }
    }
}

impl<'a, C> Id<'a, C> {
    /// Creates a new `Id` with the given name and context.
    #[inline]
    pub fn new_with_context(name: &'a str, ctx: C) -> Self {
        Id { name: name, ctx: ctx }
    }
}

impl<'a, C> Display for Id<'a, C> {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.name)
    }
}

/*
impl<'a, C> AsmDisplay for Id<'a, C> {
    #[inline]
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle) -> fmt::Result {
        s.write_id(f, self)
    }
}
*/

// -----------------------------------------------------------------------------

/*
use aex::asm::{AsmDisplay, AsmStyle};

// Just a stub for now.

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Expr<'a> {
    // Atomic
    Int(u32),
    Str(&'a str),
}

impl<'a> AsmDisplay for Expr<'a> {
    fn fmt(&self, f: &mut Formatter, s: &AsmStyle) -> fmt::Result {
        match *self {
            Expr::Int(n) => write!(f, "{}", n),
            Expr::Str(s) => write!(f, "{}", s),
        }
    }
}
*/

