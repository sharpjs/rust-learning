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

// -----------------------------------------------------------------------------

/// Trait for types that are formattable as source code.
pub trait Code: Node {
    /// Formats the object as source code in the given style.
    fn fmt<S: Style<Self::Context> + ?Sized>
          (&self, f: &mut Formatter, s: &S, p: Prec) -> fmt::Result;
}

// -----------------------------------------------------------------------------

/// A code-formattable object with the additional data required for formatting.
#[derive(Clone, Copy, Debug)]
pub struct Styled<'a,
                  T: 'a + Code              + ?Sized,
                  S: 'a + Style<T::Context> + ?Sized>(
    /// Assembly-formattable value.
    pub &'a T,

    /// Assembly code style.
    pub &'a S,

    /// Precedence level of containing code.
    pub Prec,
);

impl<'a, T, S> Styled<'a, T, S>
where T: Code              + ?Sized,
      S: Style<T::Context> + ?Sized {

    /// Formats the value using the given formatter.
    #[inline]
    pub fn new(value: &'a T, style: &'a S) -> Self {
        Styled(value, style, Prec::Statement)
    }
}

impl<'a, T, S> Display for Styled<'a, T, S>
where T: Code              + ?Sized,
      S: Style<T::Context> + ?Sized {

    /// Formats the value using the given formatter.
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Styled(value, style, prec) = *self;
        value.fmt(f, style, prec)
    }
}

// -----------------------------------------------------------------------------

/// An assembly code style.
pub trait Style<C> : Debug {
    /// Writes an identifier to the given formatter in this assembly style.
    fn write_id(&self, f: &mut Formatter, id: &Id<C>) -> fmt::Result {
        f.write_str(id.name)
    }

    /// Writes an integer literal to the given formatter in this assembly style.
    fn write_int(&self, f: &mut Formatter, num: &Int<C>) -> fmt::Result {
        write!(f, "{}", num.value)
    }

    /// Writes a register to the given formatter in this assembly style.
    fn write_reg(&self, f: &mut Formatter, reg: &Reg<C>) -> fmt::Result {
        f.write_str(reg.name)
    }

    /// Writes a unary expression to the given formatter in this assembly style.
    fn write_unary(&self, f: &mut Formatter, expr: &Unary<C>) -> fmt::Result {
        use aex::ast::Fixity::*;

        let subexpr = Styled(&*expr.expr, self, expr.precedence());

        match expr.op.fixity() {
            Prefix  => write!(f, "{}{}", expr.op, subexpr),
            Postfix => write!(f, "{}{}", subexpr, expr.op),
        }
    }

    /// Writes a binary expression to the given formatter in this assembly style.
    fn write_binary(&self, f: &mut Formatter, expr: &Binary<C>) -> fmt::Result {
        let p   = expr.precedence();
        let lhs = Styled(&*expr.lhs, self, p);
        let rhs = Styled(&*expr.rhs, self, p);

        write!(f, "({} {} {})", lhs, expr.op, rhs)
    }
}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DefaultStyle;
    impl<C> Style<C> for DefaultStyle { }

    #[test]
    fn write_id() {
        let i = Id::new("a");
        let s = format!("{}", Styled::new(&i, &DefaultStyle));
        assert_eq!(s, "a");
    }

    #[test]
    fn write_num() {
        let i = Int::from(42);
        let s = format!("{}", Styled::new(&i, &DefaultStyle));
        assert_eq!(s, "42");
    }

    #[test]
    fn write_reg() {
        let i = Reg::new("a");
        let s = format!("{}", Styled::new(&i, &DefaultStyle));
        assert_eq!(s, "a");
    }
}

