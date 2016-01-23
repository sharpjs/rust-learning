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
use num::BigInt;

use aex::ast::Expr;
use aex::pos::Pos;

use super::Context;
use super::eval::{TypeA, TypeForm, Contains, check_types_compat};

// -----------------------------------------------------------------------------
// Loc - a machine location

pub trait Loc<'a, Mode>: Display {
    fn mode(&self) -> Mode;

    fn new_const(Expr<'a>) -> Self;
    fn is_const(&self) -> bool;
    fn to_const( self) -> Expr<'a>;
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

pub type OpBySelTable_<Op> =
    &'static [(
        &'static str,
        &'static Op
    )];

macro_rules! ops {
    {$(
        ($($n:ident),+)
            =>  $family:ident ()
            + $const_op:ident ()
            +   $asm_op:ident ($write:ident, $ret:ident);
    )*}
    => {$(
        pub trait $family<'a>: 'static {
            type Loc:  'a + Loc<'a, Self::Mode>;
            type Mode: 'static;

            fn const_op(&self)
                -> &$const_op;

            fn asm_ops_by_sel(&self)
                -> OpBySelTable_<$asm_op<Self::Mode>>;

            fn asm_op_by_loc(&self, $($n: &Self::Loc),+)
                -> Option<&'static $asm_op<Self::Mode>>;

            fn invoke<'b>(
                      &self,
                      sel: Option<&str>,
                      $($n: Operand<'a, Self::Loc>),+,
                      pos: Pos<'a>,
                      ctx: &mut Context<'b, 'a>)
                      -> Result<Operand<'a, Self::Loc>, ()> {

                let op = match sel {
                    Some(sel) => {
                        self.asm_ops_by_sel()
                            .iter()
                            .find(|e| e.0 == sel)
                            .map(|&(_, op)| op)
                    },
                    _ => {
                        if true $(&& $n.loc.is_const())+ {
                            let ($($n),+,) = ($(ConstOperand {
                                expr: $n.loc.to_const(),
                                ty:   $n.ty,
                            }),+,);
                            let o = try!(
                                self.const_op().invoke($($n),+, pos, ctx)
                            );
                            return Ok(Operand {
                                loc: Self::Loc::new_const(o.expr),
                                ty:  o.ty,
                                pos: pos,
                            });
                        } else {
                            self.asm_op_by_loc($(&$n.loc),+)
                        }
                    },
                };

                match op {
                    Some(op) => {
                        op.invoke($($n),+, pos, ctx)
                    }
                    None => {
                        ctx.out.log.err_no_op_for_selector(pos);
                        Err(())
                    }
                }
            }
        }

        // Operation on constant expressions
        pub struct $const_op {
            pub check_types: for<'a> fn($($n: TypeA<'a>),+) -> Option<TypeA<'a>>,
            pub eval_int:            fn($($n: BigInt   ),+) -> BigInt,
            pub eval_float:          fn($($n: f64      ),+) -> f64,
            pub eval_expr:   for<'a> fn($($n: Expr<'a> ),+) -> Expr<'a>,
        }

        impl $const_op {
            pub fn invoke<'a, 'b>(
                          &self,
                          $($n: ConstOperand<'a>),+,
                          pos: Pos<'a>,
                          ctx: &mut Context<'b, 'a>)
                          -> Result<ConstOperand<'a>, ()> {

                // Type check
                let ty = match (self.check_types)($($n.ty),+) {
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
                        let n = (self.eval_int)($($n),+);

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
                        (self.eval_expr)($($n),+)
                    }
                };

                // Cast to checked type
                Ok(ConstOperand { expr: expr, ty: ty })
            }
        }

        // Operation on machine location(s)
        pub struct $asm_op<M: 'static> {
            pub opcodes:       OpTable,
            pub default_width: u8,

            pub check_modes:
                fn($($n: M),+) -> bool,

            pub check_types:
                for<'a> fn($($n: TypeA<'a>),+) -> Option<TypeA<'a>>,

            pub check_forms:
                fn($($n: TypeForm),+, TypeForm, u8) -> Option<u8>,
        }

        impl<M: 'static> $asm_op<M> {
            pub fn invoke<'a, 'b, L>(
                      &self,
                      $($n: Operand<'a, L>),+,
                      pos: Pos<'a>,
                      ctx: &mut Context<'b, 'a>)
                      -> Result<Operand<'a, L>, ()>
                      where L: 'a + Loc<'a, M> {

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
                let width = self.default_width;
                let width = (self.check_forms)($($n.ty.form),+, ty.form, width);
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
    )*}
}

ops! {
    // One for each arity
    (a      ) => OpFamily1() + ConstOp1() + AsmOp1(write_op_1, a);
    (a, b   ) => OpFamily2() + ConstOp2() + AsmOp2(write_op_2, b);
    (a, b, c) => OpFamily3() + ConstOp3() + AsmOp3(write_op_3, c);
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

use std::ops::Add;

pub static ADD_CONST: ConstOp2 = ConstOp2 {
    check_types: check_types_compat,
    eval_int:    BigInt::add,
    eval_float:  f64   ::add,
    eval_expr:   expr_add,
};

fn expr_add<'a>(a: Expr<'a>, b: Expr<'a>) -> Expr<'a> {
    Expr::Add(Box::new(a), Box::new(b), None)
}

