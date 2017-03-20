// Code Output Formatting
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

use std::fmt::{self, Debug, Display, Formatter};
use aex::ast::*;

pub mod att;
pub mod intel;
pub mod mit;

pub use self::att::*;
pub use self::intel::*;
pub use self::mit::*;

// Context          trait for thing with a context
// Style            trait for output language; strategy
// Styled           node + style + prec; adapter?; makes node formattable
// Code             trait for fmt
// Node?            trait to get context type
// ToStyled         trait to get style type

// -----------------------------------------------------------------------------

pub trait Style<C>: Debug {
    fn lift(&self) -> &Style<C>;
}

// -----------------------------------------------------------------------------

pub trait ToStyled: Context {
    fn styled<'a>(&'a self, style: &'a Style<Self::Context>) -> Styled<'a, Self>;
}

impl<T> ToStyled for T where T: Context {
    #[inline(always)]
    fn styled<'a>(&'a self, style: &'a Style<Self::Context>) -> Styled<'a, Self> {
        //Styled::new(self, style)
        panic!()
    }
}

// -----------------------------------------------------------------------------


/// A code-formattable object with the additional data necessary for formatting.
#[derive(Clone, Copy, Debug)]
pub struct Styled<'a, T: 'a + Context + ?Sized> {
    /// Code-formattable object.
    pub node: &'a T,

    /// Code style.
    pub style: &'a Style<T::Context>,

    /// Surrounding precedence level.
    pub prec: Prec,
}


impl<'a, T> Styled<'a, T> where T: 'a + Context + ?Sized {

    /// Constructs a new `Styled` with the given node and style.
    #[inline]
    pub fn new(node: &'a T, style: &'a Style<T::Context>) -> Self {
        Styled {
            node:  node,
            style: style,
            prec:  Prec::Statement
        }
    }

    /// Maps a `Styled<T>` to `Styled<U>` by applying a function to the
    /// contained node.
    pub fn map<F, U>(&self, f: F, prec: Prec) -> Styled<U>
    where F: FnOnce(&T) -> &U,
          U: Context<Context=T::Context> {
        Styled {
            node:  f(self.node),
            style: self.style,
            prec:  prec,
        }
    }
}

// -----------------------------------------------------------------------------

/// A type formattable as code.
pub trait Code: Context {
    fn fmt(&self, f: &mut Formatter, s: &Style<Self::Context>, p: Prec) -> fmt::Result;
}

impl<'a, C> Code for Id<'a, C> {
    fn fmt(&self, f: &mut Formatter, s: &Style<C>, p: Prec) -> fmt::Result {
        panic!()
    }
}

impl<'a, T> Display for Styled<'a, T> where T: Code {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.node.fmt(f, self.style, self.prec)
    }
}

