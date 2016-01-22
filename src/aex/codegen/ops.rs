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
use super::eval::{TypeA, TypeForm, Contains, check_types_compat};

// -----------------------------------------------------------------------------
// Loc - a machine location

pub trait Loc<'a, Mode>: Display {
    fn mode(&self) -> Mode;
    fn as_const(&self) -> Option<&Expr<'a>>;
    fn new_const(Expr<'a>) -> Self;
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

pub trait BinaryOpFamily<L, M: 'static> {
    fn by_sel(&self) -> OpBySelTable <   M>;
    fn by_loc(&self) -> OpByLocFn    <L, M>;
}

pub type OpBySelTable<M: 'static> =
    &'static [(
        &'static str,
        &'static BinaryOp<M>
    )];

pub type OpByLocFn<L, M: 'static> =
    fn(&L, &L) -> &'static BinaryOp<M>;

pub trait BinaryOpFamilyInvoke<L, M: 'static>: BinaryOpFamily<L, M> {
    fn invoke<'a, 'b>(
              &self,
              sel: Option<&str>,
              src: Operand<'a, L>,
              dst: Operand<'a, L>,
              ctx: &mut Context<'b, 'a>)
              -> Result<Operand<'a, L>, ()>
              where L: 'a + Loc<'a, M> + Display;
}

impl<L, M: 'static, F> BinaryOpFamilyInvoke<L, M> for F
    where F: BinaryOpFamilyInvoke<L, M> {

    fn invoke<'a, 'b>(
              &self,
              sel: Option<&str>,
              src: Operand<'a, L>,
              dst: Operand<'a, L>,
              ctx: &mut Context<'b, 'a>)
              -> Result<Operand<'a, L>, ()>
              where L: 'a + Loc<'a, M> + Display {

        let op = match sel {
            None => {
                Some(self.by_loc()(&src.loc, &dst.loc))
            },
            Some(sel) => {
                self.by_sel().iter()
                    .find(|e| e.0 == sel)
                    .map(|&(_, op)| op)
            },
        };

        match op {
            Some(op) => {
                op.invoke(src, dst, Pos::bof("?"), ctx)
            }
            None => {
                ctx.out.log.err_no_op_for_selector(src.pos);
                Err(())
            }
        }
    }
}

// -----------------------------------------------------------------------------
// op! - an opcode, generic over arity

macro_rules! op {
    { $name:ident ( $($n:ident),+ ) => $write:ident, $ret:ident } => {

        // Op - an opcode, with variants dispatched by size
        pub struct $name<M> {
            pub opcodes:       OpTable,
            pub default_width: u8,

            pub check_modes:
                fn($($n: M),+) -> bool,

            pub check_types:
                for<'a> fn($($n: TypeA<'a>),+) -> Option<TypeA<'a>>,

            pub check_form:
                fn(/*$($n: TypeForm),+,*/ TypeForm, u8) -> Option<u8>,

            //pub eval:
            //    Option<for<'a> fn($($n: Operand<'a, L>),+) -> Operand<'a, L>>,
        }

        impl<M> $name<M> {
            pub fn invoke<'a, 'b, L>(
                          &self,
                          $($n: Operand<'a, L>),+,
                          pos: Pos<'a>,
                          ctx: &mut Context<'b, 'a>)
                          -> Result<Operand<'a, L>, ()>
                          where L: 'a + Loc<'a, M> + Display {

                // Mode check
                let ok = (self.check_modes)($($n.loc.mode()),+);
                if !ok {
                    ctx.out.log.err_no_op_for_addr_modes(pos);
                    return Err(());
                }

                // Type check
                let ty = (self.check_types)($($n.ty),+);
                let ty = match ty {
                    Some(ty) => ty,
                    None     => {
                        ctx.out.log.err_incompatible_types(pos);
                        return Err(());
                    }
                };

                // Form check
                let width = (self.check_form)(/*$($n.ty.form),+,*/ ty.form, self.default_width);
                let width = match width {
                    Some(w) => w,
                    None    => {
                        ctx.out.log.err_no_op_for_operand_types(pos);
                        return Err(());
                    }
                };

                // Opcode select
                let op = match select_op(width, self.opcodes) {
                    Some(op) => op,
                    None     => {
                        ctx.out.log.err_no_op_for_operand_sizes(pos);
                        return Err(());
                    }
                };

                // Emit
                ctx.out.asm.$write(op, $(&$n),+);

                // Cast result to checked type
                Ok(Operand { ty: ty, .. $ret })
            }
        }
    }
}

op! { UnaryOp   (a      ) => write_op_1, a }
op! { BinaryOp  (a, b   ) => write_op_2, b }
op! { TernaryOp (a, b, c) => write_op_3, c }

// -----------------------------------------------------------------------------
// same as above, but let's do it as a trait

pub struct ConstOperand<'a> {
    pub expr: Expr <'a>,
    pub ty:   TypeA<'a>,
}

use num::BigInt;

macro_rules! ops {
    {$(
        ($($n:ident),+)
            => $const_op:ident ()
            +    $asm_op:ident ($write:ident, $ret:ident);
    )*}
    => {$(
        // Operation on constant expressions
        pub trait $const_op<'a> {
            fn check_types($($n: TypeA<'a>),+) -> Option<TypeA<'a>>;

            fn eval_int   ($($n: BigInt  ),+) -> BigInt;
            fn eval_float ($($n: f64     ),+) -> f64;
            fn eval_expr  ($($n: Expr<'a>),+) -> Expr<'a>;

            fn invoke<'b>(
                      &self,
                      $($n: ConstOperand<'a>),+,
                      pos: Pos<'a>,
                      ctx: &mut Context<'b, 'a>)
                      -> Result<ConstOperand<'a>, ()> {

                // Type check
                let ty = match Self::check_types($($n.ty),+) {
                    Some(ty) => ty,
                    None     => {
                        ctx.out.log.err_incompatible_types(pos);
                        return Err(());
                    }
                };

                // Evaluate
                let expr = match ($($n.expr),+,) {
                    ($(Expr::Int($n)),+,) => {
                        // Value computable now
                        // Compute value
                        let n = Self::eval_int($($n),+);

                        // Value check
                        if ty.contains(&n) == Some(false) {
                            ctx.out.log.err_value_out_of_range(pos);
                            return Err(());
                        }

                        // Yield reduced expression
                        Expr::Int(n)
                    }
                    ($($n),+,) => {
                        // Value not computable now
                        // Leave computation to assembler/linker
                        Self::eval_expr($($n),+)
                    }
                };

                // Cast to checked type
                Ok(ConstOperand { expr: expr, ty: ty })
            }
        }

        // Operation on machine location(s)
        pub trait $asm_op<'a> {
            type Loc:  'a + Loc<'a, Self::Mode>;
            type Mode: 'static;

            fn opcodes()       -> OpTable;
            fn default_width() -> u8;

            fn check_modes($($n: Self::Mode),+              ) -> bool;
            fn check_types($($n: TypeA<'a> ),+              ) -> Option<TypeA<'a>>;
            fn check_forms($($n: TypeForm  ),+, TypeForm, u8) -> Option<u8>;

            fn invoke<'b>(
                      &self,
                      $($n: Operand<'a, Self::Loc>),+,
                      pos: Pos<'a>,
                      ctx: &mut Context<'b, 'a>)
                      -> Result<Operand<'a, Self::Loc>, ()> {

                // Mode check
                let ok = Self::check_modes($($n.loc.mode()),+);
                if !ok {
                    ctx.out.log.err_no_op_for_addr_modes(pos);
                    return Err(());
                }

                // Type check
                let ty = Self::check_types($($n.ty),+);
                let ty = match ty {
                    Some(ty) => ty,
                    None     => {
                        ctx.out.log.err_incompatible_types(pos);
                        return Err(());
                    }
                };

                // Form check
                let width = Self::default_width();
                let width = Self::check_forms($($n.ty.form),+, ty.form, width);
                let width = match width {
                    Some(w) => w,
                    None    => {
                        ctx.out.log.err_no_op_for_operand_types(pos);
                        return Err(());
                    }
                };

                // Opcode select
                let op = match select_op(width, Self::opcodes()) {
                    Some(op) => op,
                    None     => {
                        ctx.out.log.err_no_op_for_operand_sizes(pos);
                        return Err(());
                    }
                };

                // Emit
                ctx.out.asm.$write(op, $(&$n),+);

                // Cast result to checked type
                Ok(Operand { ty: ty, .. $ret })
            }
        }
    )*}
}

ops! {
    // One for each arity
    (a      ) => ConstOp1() + AsmOp1(write_op_1, a);
    (a, b   ) => ConstOp2() + AsmOp2(write_op_2, b);
    (a, b, c) => ConstOp3() + AsmOp3(write_op_3, c);
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
// Constant operations

struct ConstAdd;

impl<'a> ConstOp2<'a> for ConstAdd {
    fn check_types(a: TypeA<'a>, b: TypeA<'a>) -> Option<TypeA<'a>> {
        check_types_compat(a, b)
    }

    fn eval_int(a: BigInt, b: BigInt) -> BigInt {
        a + b
    }

    fn eval_float(a: f64, b: f64) -> f64 {
        a + b
    }

    fn eval_expr(a: Expr<'a>, b: Expr<'a>) -> Expr<'a> {
        Expr::Add(Box::new(a), Box::new(b), None)
    }
}

