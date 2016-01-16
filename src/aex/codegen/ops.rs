// Operation Helpers
//
// This file is part of AEx.
// Copyright (C) 2016 Jeffrey Sharp
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

use aex::ast::Expr;
use aex::pos::Pos;

use super::Context;
use super::eval::{TypeA, TypeForm, Contains};

// -----------------------------------------------------------------------------
// Loc - a machine location

pub trait Loc<'a, Mode> {
    fn mode(&self) -> Mode;
    fn as_const(&self) -> Option<&Expr<'a>>;
}

// -----------------------------------------------------------------------------
// Operand - a machine location with its analyzed type and source position

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Operand<'a, L: 'a + Display> {
    pub loc: L          ,   // Machine location
    pub ty:  TypeA <'a> ,   // Analyzed type
    pub pos: Pos   <'a> ,   // Source position
}

impl<'a, L: 'a + Display> Operand<'a, L> {
    pub fn new(loc: L, ty: TypeA<'a>, pos: Pos<'a>) -> Self {
        Operand { loc: loc, ty: ty, pos: pos }
    }
}

impl<'a, L: 'a + Display> Display for Operand<'a, L> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.loc, f)
    }
}

// -----------------------------------------------------------------------------
// BinaryOpFamily - a set of binary opcodes dispatched by selector or type form

pub struct BinaryOpFamily<L, M: 'static> {
    pub by_sel: OpBySelTable <   M>,
    pub by_loc: OpByLocFn    <L, M>,
}

pub type OpBySelTable<M: 'static> =
    &'static [(
        &'static str,
        &'static BinaryOp<M>
    )];

pub type OpByLocFn<L, M: 'static> =
    fn(&L, &L) -> &'static BinaryOp<M>;

impl<L, M> BinaryOpFamily<L, M> {
    pub fn invoke<'a, 'b>(
                  &self,
                  sel: Option<&str>,
                  src: Operand<'a, L>,
                  dst: Operand<'a, L>,
                  ctx: &mut Context<'b, 'a>)
                  -> Result<Operand<'a, L>, ()>
                  where L: 'a + Loc<'a, M> + Display {

        let op = match sel {
            None => {
                Some((self.by_loc)(&src.loc, &dst.loc))
            },
            Some(sel) => {
                self.by_sel.iter()
                    .find(|e| e.0 == sel)
                    .map(|&(_, op)| op)
            },
        };

        match op {
            Some(op) => {
                op.invoke(src, dst, ctx)
            }
            None => {
                ctx.out.log.err_no_op_for_selector(src.pos);
                Err(())
            }
        }
    }
}

// -----------------------------------------------------------------------------
// BinaryOp - a binary opcode, with variants dispatched by size

pub struct BinaryOp<M> {
    pub opcodes:        OpTable,
    pub default_width:  u8,
    pub check_modes:    fn(Mode, Mode) -> bool,
    pub check_types:    for<'a> fn(TypeA<'a>, TypeA<'a>) -> Option<TypeA<'a>>,
    pub check_form:     fn(TypeForm) -> Option<u8>,
}

impl<Mode> BinaryOp<Mode> {
    pub fn invoke<'a, 'b, L: 'a + Loc<'a, Mode> + Display>(
              &self,
              src: Operand<'a, L>,
              dst: Operand<'a, L>,
              ctx: &mut Context<'b, 'a>)
              -> Result<Operand<'a, L>, ()> {

        // Mode check
        let ok = (self.check_modes)(src.loc.mode(), dst.loc.mode());
        if !ok {
            ctx.out.log.err_no_op_for_addr_modes(src.pos);
            return Err(());
        }

        // Type check
        let ty = (self.check_types)(dst.ty, src.ty);
        let ty = match ty {
            Some(ty) => ty,
            None     => {
                ctx.out.log.err_incompatible_types(src.pos);
                return Err(());
            }
        };

        // Form check
        let width = (self.check_form)(ty.form);
        let width = match width {
            Some(w) => w,
            None    => {
                ctx.out.log.err_no_op_for_operand_types(src.pos);
                return Err(());
            }
        };

        // Opcode select
        let op = match select_op(width, self.opcodes) {
            Some(op) => op,
            None     => {
                ctx.out.log.err_no_op_for_operand_sizes(src.pos);
                return Err(());
            }
        };

        // Value check
        if let Some(ref expr) = src.loc.as_const() {
            if src.ty.form.contains(expr) == Some(false) {
                ctx.out.log.err_value_out_of_range(src.pos);
                return Err(());
            }
        }

        // Emit
        ctx.out.asm.write_op_2(op, &src, &dst);

        // Return operand cast to checked type
        Ok(Operand { ty: ty, .. dst })
    }
}

// -----------------------------------------------------------------------------
// OpTable - a table mapping widths to opcodes

pub type OpTable = &'static [(u8, &'static str)];

fn select_op(ty_width: u8, ops: OpTable) -> Option<&'static str> {
    for &(op_width, op) in ops {
        if op_width == ty_width { return Some(op) }
    }
    None
}

// -----------------------------------------------------------------------------

