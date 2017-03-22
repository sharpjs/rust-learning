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

// -----------------------------------------------------------------------------
// Latest idea: Same AST for aex input and asm output.  Output AST is just a
// subset that is Codeable.  This lets me develop just an assembler and
// then bolt-on the aex features.  Possible?  Don't know.
//
// Might need two Expr types then:
//   Expr => traditional assembler expressions, constant at runtime
//   Flow => aex quasi-expressions that become assembly code
// -----------------------------------------------------------------------------

mod binary;
mod expr;
mod id;
mod int;
//mod operand;
mod prec;
mod reg;
mod unary;

pub use self::binary::*;
pub use self::expr::*;
pub use self::id::*;
pub use self::int::*;
//pub use self::operand::*;
pub use self::prec::*;
pub use self::reg::*;
pub use self::unary::*;

/// An AST node.
pub trait Node {

    /// Type of the context value.
    type Context;

    /// Gets the context value.
    fn context(&self) -> &Self::Context;
}

