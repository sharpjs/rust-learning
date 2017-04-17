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
          (&self, f: &mut Formatter, s: &S) -> fmt::Result;
}

// -----------------------------------------------------------------------------

/// Trait for types that can be converted to something code-formattable.
pub trait ToCode<A=()> {
    /// Type of the code-formattable value.
    type Output: Code<Ann=A>;

    /// Converts to a code-formattable value with the given annotation.
    fn to_code(&self, ann: A) -> Self::Output;
}

// -----------------------------------------------------------------------------

/// A code-formattable value paired with a code style.
#[derive(Clone, Copy, Debug)]
pub struct Styled<'a,
                  T: 'a + Code          + ?Sized,
                  S: 'a + Style<T::Ann> + ?Sized>(

    /// Code-formattable value.
    pub &'a T,

    /// Code style.
    pub &'a S
);

impl<'a, T, S> Display for Styled<'a, T, S>
where T: Code          + ?Sized,
      S: Style<T::Ann> + ?Sized {

    /// Formats the value using the given formatter.
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Styled(value, style) = *self;
        value.fmt(f, style)
    }
}

impl<'a, T, S> Styled<'a, T, S>
where T: Code + HasPrec + ?Sized,
      S: Style<T::Ann>  + ?Sized {

    /// Formats the value using the given formatter, grouped in parentheses if
    /// required by operator precedence.
    fn fmt_grouped(&self, f: &mut Formatter, outer: Prec, if_eq: bool) -> fmt::Result {
        if self.0.prec().should_group(outer, if_eq) {
            write!(f, "({})", self)
        } else {
            write!(f, "{}", self)
        }
    }
}

// -----------------------------------------------------------------------------

pub trait ToStyled: Code {
    /// Applies a code style to the given value.  The returned value is
    /// ready for formatting.
    fn styled<'a, S>(&'a self, style: &'a S) -> Styled<'a, Self, S>
    where S: Style<Self::Ann> + ?Sized {
        Styled(self, style)
    }
}

impl<T> ToStyled for T where T: Code + ?Sized {}

// -----------------------------------------------------------------------------

/// A code formatting style.
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
    fn write_unary(&self, f: &mut Formatter, expr: &Unary<A>) -> fmt::Result {
        use aex::ast::Assoc::*;

        let prec   = expr.op.prec();
        let prefix = expr.op.assoc() == Right;

        if prefix { write!(f, "{}", expr.op)?; }

        expr.expr.styled(self).fmt_grouped(f, prec, false)?;

        if !prefix { write!(f, "{}", expr.op)?; }

        Ok(())
    }

    /// Writes a binary expression to the given formatter in this code style.
    fn write_binary(&self, f: &mut Formatter, expr: &Binary<A>) -> fmt::Result {
        use aex::ast::Assoc::*;

        let prec  = expr.op.prec();
        let assoc = expr.op.assoc();

        expr.lhs.styled(self).fmt_grouped(f, prec, assoc != Left)?;

        write!(f, " {} ", expr.op)?;

        expr.rhs.styled(self).fmt_grouped(f, prec, assoc != Right)
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
        let s = format!("{}", i.styled(&DefaultStyle));
        assert_eq!(s, "a");
    }

    #[test]
    fn write_num() {
        let i = Int::from(42);
        let s = format!("{}", i.styled(&DefaultStyle));
        assert_eq!(s, "42");
    }

    #[test]
    fn write_reg() {
        let i = Reg::new("a");
        let s = format!("{}", i.styled(&DefaultStyle));
        assert_eq!(s, "a");
    }

    #[test]
    fn write_binary_1() {
        let e = Binary::new(
            BinaryOp::Add,
            Expr::Id(Id::new("a")),
            Expr::Id(Id::new("b"))
        );

        let s = format!("{}", e.styled(&DefaultStyle));
        assert_eq!(s, "a + b");
    }

    #[test]
    fn write_binary_2() {
        let lhs = Binary::new(BinaryOp::Add, Expr::Id(Id::new("a")), Expr::Id(Id::new("b")));
        let rhs = Binary::new(BinaryOp::Add, Expr::Id(Id::new("c")), Expr::Id(Id::new("d")));
        let e   = Binary::new(BinaryOp::Add, Expr::Binary(lhs), Expr::Binary(rhs));
        let s   = format!("{}", e.styled(&DefaultStyle));
        assert_eq!(s, "a + b + (c + d)");
    }
}

