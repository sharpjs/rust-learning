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

//use aex::ast::*;
//use aex::codegen::Context;
use aex::codegen::eval::*;
use aex::codegen::ops;

use super::loc::*;

pub type AsmOp1         = ops::AsmOp1<Mode>;
pub type AsmOp2         = ops::AsmOp2<Mode>;
pub type AsmOp3         = ops::AsmOp3<Mode>;

//type BinaryOp       = ops::AsmOp2     <             Mode>;
//type OpBySelTable   = ops::OpBySelTable <             Mode>;
////type OpByLocFn <'a> = ops::OpByLocFn    <    Loc<'a>, Mode>;
pub type Operand   <'a> = ops::Operand      <'a, Loc<'a>      >;
//type OpTable        = ops::OpTable;

const BYTE: u8 =  8;
const WORD: u8 = 16;
const LONG: u8 = 32;

//// -----------------------------------------------------------------------------
//
//pub struct Evaluator;
//
//impl Eval for Evaluator {
//    #[inline]
//    #[allow(unused_must_use)]
//    fn eval<'cg, 'str>(
//            &self,
//            expr: &Expr<'str>,
//            ctx:  &mut Context<'cg, 'str>) {
//
//        // Delegate to the real `eval` and ignore its result.
//        Self::eval(expr, ctx);
//    }
//}
//
//impl Evaluator {
//    fn eval<'cg, 'str>(
//            expr: &Expr<'str>,
//            ctx:  &mut Context<'cg, 'str>)
//            ->    Result<Operand<'str>, ()> {
//
//        macro_rules! op {
//            ($op:ident [$sel:ident] $($arg:ident),*) => {{
//                $(
//                    let $arg = try!(Self::eval($arg, ctx));
//                )*
//                $op.invoke($sel, $($arg),*, ctx)
//            }};
//        }
//
//        match *expr {
//            Expr::Add      (ref d, ref s, k) => op!(ADD [k] s, d),
//            Expr::Subtract (ref d, ref s, k) => op!(SUB [k] s, d),
//            _ => {
//                // TODO: Pos in expression
//                //ctx.out.log.err_no_op_for_expression(pos);
//                Err(())
//            }
//        }
//    }
//}

// -----------------------------------------------------------------------------

pub struct AddFamily;

impl<'a> ops::OpFamily2<'a> for AddFamily {
    type Loc  = Loc<'a>;
    type Mode = Mode;

    fn const_op(&self) -> &ops::ConstOp2 {
        &ops::ADD_CONST
    }

    fn asm_ops_by_sel(&self) -> ops::OpBySelTable<AsmOp2> {
        ADD_BY_SEL
    }

    fn asm_op_by_loc(&self, s: &Loc<'a>, d: &Loc<'a>) -> Option<&'static AsmOp2> {
        match (s.mode(), d.mode()) {
            (M_Imm,  M_Imm )                            => Some(&ADDA),
            (_,      M_Addr) if s.is(M_Src)             => Some(&ADDA),
            (M_Imm,  _     ) if d.is(M_Dst) && s.is_q() => Some(&ADDA),
            (M_Imm,  M_Data)                            => Some(&ADDA),
            (M_Data, _     ) if d.is(M_Dst)             => Some(&ADDA),
            (_,      M_Data) if s.is(M_Src)             => Some(&ADDA),
            _                                           => Some(&ADDA),
        }
    }
}

static ADD_BY_SEL: ops::OpBySelTable<AsmOp2> = &[
    ("a", &ADDA)
];

// -----------------------------------------------------------------------------

static ADDA: AsmOp2 = AsmOp2 {
    opcodes:        &[(LONG, "adda.l")],
    default_width:  LONG,
    check_modes:    check_modes_src_addr,
    check_types:    check_types_compat,
    check_forms:    check_forms_inty,
};

static SUBA: AsmOp2 = AsmOp2 {
    opcodes:        &[(LONG, "suba.l")],
    default_width:  LONG,
    check_modes:    check_modes_src_addr,
    check_types:    check_types_compat,
    check_forms:    check_forms_inty,
};

static MOVEA: AsmOp2 = AsmOp2 {
    opcodes:        &[(LONG, "movea.l"),
                      (WORD, "movea.w")],
    default_width:  LONG,
    check_modes:    check_modes_src_addr,
    check_types:    check_types_compat,
    check_forms:    check_forms_inty,
};

//static EXT: UnaryOp = UnaryOp {
//    opcodes:        &[(LONG, "ext.l" ),  // word -> long
//                      (WORD, "ext.w" ),  // byte -> word
//                      (BYTE, "extb.l")], // byte -> long
//    default_width:  LONG,
//    check_modes:    check_modes_src_addr,
//    check_types:    check_types_compat,
//    check_form:     check_form_inty_extend,
//};

// -----------------------------------------------------------------------------

fn check_modes_src_addr(src: Mode, dst: Mode) -> bool {
    dst == M_Addr && mode_any(src, M_Src)
}

// -----------------------------------------------------------------------------
//
//        match sel {
//            "a" => return self.adda(x, y),
//            "d" => return self.addd(x, y),
//            "i" => return self.addi(x, y),
//            "q" => return self.addq(x, y),
//            "x" => return self.addx(x, y),
//            _   => {}
//        }
//
//        match modes {
//            (M_Imm,  M_Imm )                                    => self.addc(        x, y),
//            (_,      M_Addr) if x.loc.is(M_Src)                 => self.op2l("adda", x, y),
//            (M_Imm,  _     ) if y.loc.is(M_Dst) && x.loc.is_q() => self.op2l("addq", x, y),
//            (M_Imm,  M_Data)                                    => self.op2l("addi", x, y),
//            (M_Data, _     ) if y.loc.is(M_Dst)                 => self.op2l("add",  x, y),
//            (_,      M_Data) if x.loc.is(M_Src)                 => self.op2l("add",  x, y),
//            _                                                   => Err(())
//        }
//
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

#[cfg(test)]
mod tests {
    use num::bigint::ToBigInt;

    use aex::ast::Expr;
    use aex::codegen::Context;
    use aex::codegen::eval::*;
    use aex::codegen::ops::OpFamily2;
    use aex::output::Output;
    use aex::pos::Pos;
    use aex::scope::Scope;
    use aex::target::cf::loc::*;
    use aex::target::cf::loc::AddrReg::*;
  //use aex::target::cf::loc::DataReg::*;
    use aex::types::*;
    use super::*;

    #[test]
    fn foo() {
        let mut out = Output::new();
        let mut ctx = Context { scope: Scope::new(), out: &mut out };

        let ta  = analyze_type(U32, &ctx.scope).unwrap();
        let pos = Pos::bof("f");

        let n   = 4u8.to_bigint().unwrap();
        let src = Operand::new(Loc::Imm(Expr::Int(n)), ta, pos);
        let dst = Operand::new(Loc::Addr(A3),          ta, pos);
        let exp = Operand::new(Loc::Addr(A3),          ta, pos);

        let res = AddFamily.invoke(None, src, dst, pos, &mut ctx);

        println!("{:#?}", ctx.out);
        assert_eq!(res, Ok(exp));
    }
}

