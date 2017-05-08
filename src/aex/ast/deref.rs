// Dereference Expressions
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
use aex::ast::{Node, Expr};
use aex::ast::{HasPrec, Prec};

/// A dereference expression.
#[derive(Clone, Debug)]
pub struct Deref<'a, A = ()> {
    /// Terms that compute the effective address.
    pub terms: Vec<Expr<'a, A>>,

    /// Annotation.
    pub ann: A,
}

impl<'a> Deref<'a> {
    /// Creates a new `Deref` with the given terms and with `()` annotation.
    pub fn new<T>(terms: T) -> Self
    where T: Into<Vec<Expr<'a>>> {
        Self::new_with_ann(terms, ())
    }
}

impl<'a, A> Deref<'a, A> {
    /// Creates a new `Deref` with the given terms and annotation.
    pub fn new_with_ann<T>(terms: T, ann: A) -> Self
    where T: Into<Vec<Expr<'a, A>>> {
        Self { terms: terms.into(), ann }
    }
}

impl<'a, A> Node for Deref<'a, A> {
    /// Annotation type.
    type Ann = A;

    /// Gets the annotation for this node.
    fn ann(&self) -> &A { &self.ann }
}

impl<'a, A> HasPrec for Deref<'a, A> {
    /// Gets the operator precedence level.
    #[inline]
    fn prec(&self) -> Prec { Prec::Atomic }
}

impl<'a, A> Display for Deref<'a, A> {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        panic!()
    }
}

impl<'a, A> Code for Deref<'a, A> {
    /// Formats the value as code, using the given formatter and style.
    fn fmt<S: Style<A> + ?Sized>
          (&self, f: &mut Formatter, s: &S) -> fmt::Result {
        //s.write_unary(f, self)
        panic!()
    }
}
