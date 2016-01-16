// MCF5307 Code Generation
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

#![allow(non_upper_case_globals)]

use aex::ast::*;
use aex::codegen::Context;
use aex::codegen::eval::*;
use aex::codegen::ops;
use aex::codegen::ops::BinaryOpFamilyInvoke;

use super::loc::*;

type BinaryOp       = ops::BinaryOp     <             Mode>;
type OpBySelTable   = ops::OpBySelTable <             Mode>;
type OpByLocFn <'a> = ops::OpByLocFn    <    Loc<'a>, Mode>;
type Operand   <'a> = ops::Operand      <'a, Loc<'a>      >;
type OpTable        = ops::OpTable;

const BYTE: u8 =  8;
const WORD: u8 = 16;
const LONG: u8 = 32;

// -----------------------------------------------------------------------------

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

        macro_rules! op {
            ($op:ident [$sel:ident] $($arg:ident),*) => {{
                $(
                    let $arg = try!(Self::eval($arg, ctx));
                )*
                $op.invoke($sel, $($arg),*, ctx)
            }};
        }

        match *expr {
            Expr::Add      (ref d, ref s, k) => op!(ADD [k] s, d),
            Expr::Subtract (ref d, ref s, k) => op!(SUB [k] s, d),
            _ => {
                // TODO: Pos in expression
                //ctx.out.log.err_no_op_for_expression(pos);
                Err(())
            }
        }
    }
}

// -----------------------------------------------------------------------------

struct BinaryOpFamily {
    by_sel: OpBySelTable,
    by_loc: fn(&Loc, &Loc) -> &'static BinaryOp,
}

impl<'a> ops::BinaryOpFamily<Loc<'a>, Mode> for BinaryOpFamily {
    fn by_sel(&self) -> OpBySelTable  { self.by_sel }
    fn by_loc(&self) -> OpByLocFn<'a> { self.by_loc }
}

static ADD: BinaryOpFamily = BinaryOpFamily {
    by_sel: &[("a", &ADDA)],
    by_loc: choose_add,
};

static SUB: BinaryOpFamily = BinaryOpFamily {
    by_sel: &[("a", &SUBA)],
    by_loc: choose_sub,
};

fn choose_add(s: &Loc, d: &Loc) -> &'static BinaryOp {
    match (s.mode(), d.mode()) {
        (M_Imm,  M_Imm )                            => &ADDA,
        (_,      M_Addr) if s.is(M_Src)             => &ADDA,
        (M_Imm,  _     ) if d.is(M_Dst) && s.is_q() => &ADDA,
        (M_Imm,  M_Data)                            => &ADDA,
        (M_Data, _     ) if d.is(M_Dst)             => &ADDA,
        (_,      M_Data) if s.is(M_Src)             => &ADDA,
        _                                           => &ADDA,
    }
}

fn choose_sub(s: &Loc, d: &Loc) -> &'static BinaryOp {
    match (s.mode(), d.mode()) {
        (M_Imm,  M_Imm )                            => &SUBA,
        (_,      M_Addr) if s.is(M_Src)             => &SUBA,
        (M_Imm,  _     ) if d.is(M_Dst) && s.is_q() => &SUBA,
        (M_Imm,  M_Data)                            => &SUBA,
        (M_Data, _     ) if d.is(M_Dst)             => &SUBA,
        (_,      M_Data) if s.is(M_Src)             => &SUBA,
        _                                           => &SUBA,
    }
}

// -----------------------------------------------------------------------------

static ADDA: BinaryOp = BinaryOp {
    opcodes:        &[(LONG, "adda.l")],
    default_width:  LONG,
    check_modes:    check_modes_src_addr,
    check_types:    typecheck,
    check_form:     check_form_inty,
};

static SUBA: BinaryOp = BinaryOp {
    opcodes:        &[(LONG, "suba.l")],
    default_width:  LONG,
    check_modes:    check_modes_src_addr,
    check_types:    typecheck,
    check_form:     check_form_inty,
};

// -----------------------------------------------------------------------------

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

