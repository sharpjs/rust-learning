// MCF5307 Code Generation
//
// This file is part of AEx.
// Copyright (C) 2015 Jeffrey Sharp
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

#![allow(non_upper_case_globals)]

use aex::ast::*;
use aex::codegen::Context;
use aex::codegen::eval::{self, Eval, TypeA, TypeForm};
use aex::types::IntSpec;


use super::loc::*;

type Operand<'a> = eval::Operand<'a, Loc<'a>>;

// -----------------------------------------------------------------------------
// Evaluator

pub struct Evaluator;

impl Eval for Evaluator {
    #[inline]
    #[allow(unused_must_use)]
    fn eval<'cg, 'str>(
            &self,
            expr: &Expr<'str>,
            ctx:  &mut Context<'cg, 'str>) {

        // Delegate to the real `eval` and ignore its result.
        Self::eval(expr, ctx);
    }
}

impl Evaluator {
    fn eval<'cg, 'str>(
            expr: &Expr<'str>,
            ctx:  &mut Context<'cg, 'str>)
            ->    Result<Operand<'str>, ()> {

        match *expr {
            Expr::Add(ref src, ref dst, sel) => {
                let src = try!(Self::eval(src, ctx));
                let dst = try!(Self::eval(dst, ctx));
                Self::add(src, dst, sel.unwrap_or(""), ctx)
            },
            // Subtract, etc...
            _ => {
                Err(())
            }
        }
    }

    fn add<'cg, 'str>(
           src: Operand<'str>,
           dst: Operand<'str>,
           sel: &str,
           ctx: &mut Context<'cg, 'str>)
           ->   Result<Operand<'str>, ()> {

        // Choose via selector
        match sel {
            ""  => {},
            "a" => return ADDA.invoke(src, dst, ctx),
        //  "d" => return self.addd(x, y),
        //  "i" => return self.addi(x, y),
        //  "q" => return self.addq(x, y),
        //  "x" => return self.addx(x, y),
            _   => {
                ctx.out.log.err_no_op_for_selector(src.pos);
                return Err(());
            }
        }

        // Choose via modes
        Ok(dst)
    }
}

fn typecheck<'a>(x: TypeA<'a>, y: TypeA<'a>) -> Option<TypeA<'a>> {
   
    // A type is compatible with itself
    //
    if x.ty as *const _ == y.ty as *const _ {
        return Some(x);
    }

    // Otherwise, two types are compatible if:
    //   - they are of the same form, and
    //   - at least one is arbitrary
    //
    match (x.form, y.form) {
        (TypeForm::Inty(xf), TypeForm::Inty(yf)) => {
            match (xf, yf) {
                (_, None) => Some(x),
                (None, _) => Some(y),
                _         => None,
            }
        },
        (TypeForm::Floaty(xf), TypeForm::Floaty(yf)) => {
            match (xf, yf) {
                (_, None) => Some(x),
                (None, _) => Some(y),
                _         => None,
            }
        },
        _ => None
    }
}

type OpTable = &'static [(u8, &'static str)];

fn select_op(ty_width: u8, ops: OpTable) -> Option<&'static str> {
    for &(op_width, op) in ops {
        if op_width == ty_width { return Some(op) }
    }
    None
}

const BYTE: u8 =  8;
const WORD: u8 = 16;
const LONG: u8 = 32;

const OPS_ADDA: OpTable = &[
    (LONG, "adda.l")
];

// -----------------------------------------------------------------------------

type ModeCheck =         fn(Mode,      Mode     ) -> bool;
type TypeCheck = for<'a> fn(TypeA<'a>, TypeA<'a>) -> Option<TypeA<'a>>;
type FormCheck =         fn(TypeForm            ) -> Option<u8>;

struct BinaryOp {
    opcodes:        OpTable,
    default_width:  u8,
    check_modes:    ModeCheck,
    check_types:    TypeCheck,
    check_form:     FormCheck,
}

impl BinaryOp {
    fn invoke<'a, 'b>(
              &self,
              src: Operand<'a>,
              dst: Operand<'a>,
              ctx: &mut Context<'b, 'a>)
              -> Result<Operand<'a>, ()> {

        // Mode check
        let ok = (self.check_modes)(src.loc.mode(), dst.loc.mode());
        if !ok {
            ctx.out.log.err_no_op_for_addr_modes(src.pos);
            return Err(());
        }

        // General type check
        let ty = (self.check_types)(dst.ty, src.ty);
        let ty = match ty {
            Some(ty) => ty,
            None => {
                ctx.out.log.err_incompatible_types(src.pos);
                return Err(());
            }
        };

        // Opcode type check
        let w = (self.check_form)(ty.form);
        let w = match w {
            Some(w) => w,
            None => {
                ctx.out.log.err_no_op_for_operand_types(src.pos);
                return Err(());
            }
        };

        // Opcode variant check
        let op = match select_op(w, self.opcodes) {
            Some(op) => op,
            None => {
                ctx.out.log.err_no_op_for_operand_sizes(src.pos);
                return Err(());
            }
        };

        // Value check
        if let Loc::Imm(ref expr) = src.loc {
            if src.ty.form.contains(expr) == Some(false) {
                ctx.out.log.err_value_out_of_range(src.pos);
                return Err(());
            }
        }

        // Emit
        ctx.out.asm.write_op_2(op, &src, &dst);

        Ok(Operand { ty: ty, .. dst })
    }
}

static ADDA: BinaryOp = BinaryOp {
    opcodes:        &[(LONG, "adda.l")],
    default_width:  LONG,
    check_modes:    check_modes_src_addr,
    check_types:    typecheck,
    check_form:     check_form_inty,
};

fn check_modes_src_addr(src: Mode, dst: Mode) -> bool {
    dst == M_Addr && mode_any(src, M_Src)
}

fn check_form_inty(form: TypeForm) -> Option<u8> {
    match form {
        TypeForm::Inty(None)    => Some(LONG),
        TypeForm::Inty(Some(s)) => Some(s.store_width),
        _                       => None
    }
}

//fn binary_op<'a, 'b>
//       (self:     &    Self,
//        src:           Operand<    'a>,
//        dst:           Operand<    'a>,
//        ctx:      &mut Context<'b, 'a>,
//        mode_ck:  &    MC,
//        type_ck:  &    TC,
//        def_wid:       u8,
//        opcodes:  &    OpTable)
//       -> Result<Operand<'a>, ()> {
//       where MC: Fn(AddrMode, AddrMode) -> bool,
//             TC: Fn(&Type,    &Type   ) -> bool
//
//}

// -----------------------------------------------------------------------------

trait Contains<T> {
    fn contains(&self, item: &T) -> Option<bool>;
    //
    // Some(true)  => item definitely     in self
    // Some(false) => item definitely not in self
    // None        => unknown
}

impl<'a> Contains<Expr<'a>> for IntSpec {
    fn contains(&self, expr: &Expr<'a>) -> Option<bool> {
        match *expr {
            Expr::Int(ref v) => {
                let ok = *v >= self.min_value()
                      && *v <= self.max_value();
                Some(ok)
            },
            _ => None,
        }
    }
}

impl<'a, T, S> Contains<T> for Option<S> where S: Contains<T> {
    fn contains(&self, item: &T) -> Option<bool> {
        match *self {
            Some(ref s) => s.contains(item),
            None        => None,
        }
    }
}

impl<'a> Contains<Expr<'a>> for TypeForm {
    fn contains(&self, expr: &Expr<'a>) -> Option<bool> {
        match *self {
            TypeForm::Inty   (s) => s.contains(expr),
            TypeForm::Floaty (s) => None,           // Don't know for now
            TypeForm::Opaque     => Some(false)     // Inexpressable?
        }
    }
}

//// -----------------------------------------------------------------------------
//// Evaluator
//
//pub struct CodeGen<'a, W: io::Write> {
//    out:  W,
//    msgs: Messages<'a>,
//}
//
//impl<'a, W> CodeGen<'a, W> where W: io::Write {
//    pub fn new(out: W) -> Self {
//        CodeGen { out: out, msgs: Messages::new() }
//    }
//
//    // This is all WIP, just idea exploration.
//
//    pub fn visit_expr(&mut self, expr: &Expr<'a>) -> Result<Operand<'a>, ()> {
//        match *expr {
//            Expr::Add(ref src, ref dst, sel) => {
//                let src = try!(self.visit_expr(src));
//                let dst = try!(self.visit_expr(dst));
//                self.add(&src, &dst, sel.unwrap_or(""))
//            },
//            Expr::Int(_) => {
//                Ok(Operand::new(Loc::Imm(expr.clone()), INT, Pos::bof("f")))
//            }
//            _ => {
//                Err(())
//            }
//        }
//    }
//
//    pub fn add(&mut self, x: &Operand<'a>, y: &Operand<'a>, sel: &str)
//                  -> Result<Operand<'a>, ()> {
//        match sel {
//            "a" => return self.adda(x, y),
//            "d" => return self.addd(x, y),
//            "i" => return self.addi(x, y),
//            "q" => return self.addq(x, y),
//            "x" => return self.addx(x, y),
//            _   => {}
//        }
//
//        let ty = try!(self.check_types(types_eq_scalar, x, y));
//
//        let modes = (x.loc.mode(), y.loc.mode());
//        match modes {
//            (M_Imm,  M_Imm )                                    => self.addc(        x, y),
//            (_,      M_Addr) if x.loc.is(M_Src)                 => self.op2l("adda", x, y),
//            (M_Imm,  _     ) if y.loc.is(M_Dst) && x.loc.is_q() => self.op2l("addq", x, y),
//            (M_Imm,  M_Data)                                    => self.op2l("addi", x, y),
//            (M_Data, _     ) if y.loc.is(M_Dst)                 => self.op2l("add",  x, y),
//            (_,      M_Data) if x.loc.is(M_Src)                 => self.op2l("add",  x, y),
//            _                                                   => Err(())
//        }
//    }
//
//    pub fn adda(&mut self, x: &Operand<'a>, y: &Operand<'a>)
//                   -> Result<Operand<'a>, ()> {
//        let modes = (x.loc.mode(), y.loc.mode());
//        match modes {
//            (_, M_Addr) if x.loc.is(M_Src) => self.op2l("adda", x, y),
//            _                              => Err(())
//        }
//    }
//
//    pub fn addd(&mut self, x: &Operand<'a>, y: &Operand<'a>)
//                   -> Result<Operand<'a>, ()> {
//        let modes = (x.loc.mode(), y.loc.mode());
//        match modes {
//            (M_Data, _     ) if y.loc.is(M_Dst) => self.op2l("add", x, y),
//            (_,      M_Data) if x.loc.is(M_Src) => self.op2l("add", x, y),
//            _                                   => Err(())
//        }
//    }
//
//    pub fn addi(&mut self, x: &Operand<'a>, y: &Operand<'a>)
//                   -> Result<Operand<'a>, ()> {
//        let modes = (x.loc.mode(), y.loc.mode());
//        match modes {
//            (M_Imm,  M_Data) => self.op2l("addi", x, y),
//            _                => Err(())
//        }
//    }
//
//    pub fn addq(&mut self, x: &Operand<'a>, y: &Operand<'a>)
//                   -> Result<Operand<'a>, ()> {
//        let modes = (x.loc.mode(), y.loc.mode());
//        match modes {
//            (M_Imm,  _) if y.loc.is(M_Dst) && x.loc.is_q() => self.op2l("addq", x, y),
//            _                                              => Err(())
//        }
//    }
//
//    pub fn addx(&mut self, x: &Operand<'a>, y: &Operand<'a>)
//                   -> Result<Operand<'a>, ()> {
//        let modes = (x.loc.mode(), y.loc.mode());
//        match modes {
//            (M_Data, M_Data) => self.op2l("addx", x, y),
//            _                => Err(())
//        }
//    }
//
//    fn addc(&mut self, x: &Operand<'a>, y: &Operand<'a>)
//                   -> Result<Operand<'a>, ()> {
//        let args = (
//            x.loc.as_expr(), y.loc.as_expr()
//        );
//        let expr = match args {
//            (&Expr::Int(ref x), &Expr::Int(ref y)) => {
//                Expr::Int(x + y)
//            },
//            (x, y) => {
//                Expr::Add(Box::new(x.clone()), Box::new(y.clone()), None)
//            }
//        };
//        Ok(Operand::new(Loc::Imm(expr), INT, x.pos))
//    }
//
//    fn op2l(&mut self, op: &str, x: &Operand<'a>, y: &Operand<'a>)
//               -> Result<Operand<'a>, ()> {
//        let t = try!(self.check_types(types_eq_scalar, x, y));
//        self.write_ins_s2(op, t, x, y);
//        Ok(y.clone())
//    }
//
//    fn write_ins_s2<A: Display, B: Display>
//                   (&mut self, op: &str, ty: &Type, a: &A, b: &B) {
//        let suf = Self::size_suffix(ty);
//        writeln!(self.out, "    {}{} {}, {}", op, suf, a, b).unwrap();
//    }
//
//    fn size_suffix(ty: &Type) -> &'static str {
//        match ty.store_width() {
//            Some(8)  => ".b",
//            Some(16) => ".w",
//            Some(32) => ".l",
//            _        => ""
//        }
//    }
//
//    fn check_types<'b>
//                  (&self,
//                   f: fn(&'b Type<'a>, &'b Type<'a>) -> Option<&'b Type<'a>>,
//                   x: &'b Operand<'a>,
//                   y: &'b Operand<'a>)
//                  -> Result<&'b Type<'a>, ()> {
//        f(&x.ty, &y.ty).ok_or(())
//    }
//}
//
//fn types_eq_scalar<'a, 'b>
//                  (x: &'b Type<'a>, y: &'b Type<'a>)
//                  -> Option<&'b Type<'a>> {
//    match (x, y) {
//        (&Type::Int(xx), &Type::Int(yy)) => match (xx, yy) {
//            _ if xx == yy => Some(x),
//            (_, None)     => Some(x),
//            (None, _)     => Some(y),
//            _             => None
//        },
//        _ => None
//    }
//}
//
//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    use std::io;
//    use num::bigint::ToBigInt;
//
//    use aex::ast::Expr;
//    use aex::pos::Pos;
//    use aex::targets::mcf5307::loc::*;
//    use aex::targets::mcf5307::loc::DataReg::*;
//  //use aex::targets::mcf5307::loc::AddrReg::*;
//    use aex::types::*;
//
//    #[test]
//    fn foo() {
//        let n   = 4u8.to_bigint().unwrap();
//        let src = Operand::new(Loc::Imm(Expr::Int(n)), U8, Pos::bof("f"));
//        let dst = Operand::new(Loc::Data(D3),          U8, Pos::bof("f"));
//
//        let mut gen = CodeGen::new(io::stdout());
//        let res = gen.add(&src, &dst, "").unwrap();
//
//        assert_eq!(dst, res);
//    }
//}

