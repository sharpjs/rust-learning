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

/// Trait for types that are formattable as code.
pub trait Code: Node {

    /// Formats the object as code in the given style.
    fn fmt<S: Style<Self::Ann> + ?Sized>
          (&self, f: &mut Formatter, s: &S, p: Prec) -> fmt::Result;
}

// -----------------------------------------------------------------------------

/// A code-formattable object with the additional data required for formatting.
#[derive(Clone, Copy, Debug)]
pub struct Styled<'a,
                  T: 'a + Code          + ?Sized,
                  S: 'a + Style<T::Ann> + ?Sized>(

    /// Code-formattable object.
    pub &'a T,

    /// Code style.
    pub &'a S,

    /// Surrounding precedence level.
    pub Prec,
);

impl<'a, T, S> Styled<'a, T, S>
where T: Code          + ?Sized,
      S: Style<T::Ann> + ?Sized {

    /// Formats the value using the given formatter and style.
    #[inline]
    pub fn new(value: &'a T, style: &'a S) -> Self {
        Styled(value, style, Prec::Statement)
    }
}

impl<'a, T, S> Display for Styled<'a, T, S>
where T: Code          + ?Sized,
      S: Style<T::Ann> + ?Sized {

    /// Formats the value using the given formatter.
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Styled(value, style, prec) = *self;
        value.fmt(f, style, prec)
    }
}

// -----------------------------------------------------------------------------

pub trait ToStyled: Code {
    fn styled<'a, S>(&'a self, style: &'a S, prec: Prec) -> Styled<'a, Self, S>
    where S: Style<Self::Ann> + ?Sized {
        Styled(self, style, prec)
    }
}

impl<T> ToStyled for T where T: Code + ?Sized {}

// -----------------------------------------------------------------------------

/// A code output style.
pub trait Style<A> : Debug {
    /// Writes an identifier to the given formatter in this code style.
    fn write_id(&self, f: &mut Formatter, id: &Id<A>) -> fmt::Result {
        f.write_str(id.name)
    }

    /// Writes an integer literal to the given formatter in this code style.
    fn write_int(&self, f: &mut Formatter, num: &Int<A>) -> fmt::Result {
        write!(f, "{}", num.value)
    }

    /// Writes a register to the given formatter in this code style.
    fn write_reg(&self, f: &mut Formatter, reg: &Reg<A>) -> fmt::Result {
        f.write_str(reg.name)
    }

    /// Writes a unary expression to the given formatter in this code style.
    fn write_unary(&self, f: &mut Formatter, expr: &Unary<A>, prec: Prec) -> fmt::Result {
        use aex::ast::Fixity::*;

        let (in_prec, my_prec) = (prec, expr.precedence());

        if in_prec > my_prec {
            write!(f, "({})", expr.styled(self, my_prec))
        } else {
            let subexpr = expr.expr.styled(self, my_prec);

            match expr.op.fixity() {
                Prefix  => write!(f, "{}{}", expr.op, subexpr),
                Postfix => write!(f, "{}{}", subexpr, expr.op),
            }
        }
    }

    /// Writes a binary expression to the given formatter in this code style.
    fn write_binary(&self, f: &mut Formatter, expr: &Binary<A>, prec: Prec) -> fmt::Result {
        let (in_prec, my_prec) = (prec, expr.precedence());

        if in_prec > my_prec {
            write!(f, "({})", expr.styled(self, my_prec))
        } else {
            let prec = expr.precedence();
            let lhs  = expr.lhs.styled(self, my_prec);
            let rhs  = expr.rhs.styled(self, my_prec);

            write!(f, "{} {} {}", lhs, expr.op, rhs)
        }
    }
}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DefaultStyle;
    impl<A> Style<A> for DefaultStyle { }

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

    #[test]
    fn write_binary_1() {
        let e = Binary::new(
            BinaryOp::Add,
            Expr::Id(Id::new("a")),
            Expr::Id(Id::new("b"))
        );

        let s = format!("{}", e.styled(&DefaultStyle, Prec::Atomic));
        assert_eq!(s, "(a + b)");

        let s = format!("{}", e.styled(&DefaultStyle, Prec::Assignment));
        assert_eq!(s, "a + b");
    }
}

