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

use std::io;
use std::fmt::{self, Display, Formatter};

use ast::*;
use mcf5307::loc::*;
use types::*;
use util::Pos;

// -----------------------------------------------------------------------------
// Operand - a machine location with its analyzed type and source position

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Operand<'a> {
    pub loc: Loc<'a>,       // Machine location
    pub ty:  &'a Type<'a>,  // Analyzed type
    pub pos: Pos,           // Source position
}

impl<'a> Operand<'a> {
    pub fn new(loc: Loc<'a>, ty: &'a Type<'a>, pos: Pos) -> Self
    {
        Operand { loc: loc, ty: ty, pos: pos }
    }
}

impl<'a> Display for Operand<'a> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.loc, f)
    }
}

// -----------------------------------------------------------------------------
// Code Generator

pub struct CodeGen<W: io::Write> {
    out: W
}

impl<W> CodeGen<W> where W: io::Write {
    pub fn new(out: W) -> Self {
        CodeGen { out: out }
    }

    // This is all WIP, just idea exploration.

    pub fn visit_expr<'a>(&mut self, expr: Box<Expr<'a>>) -> Operand<'a> {
        let expr = *expr; // https://github.com/rust-lang/rust/issues/16223
        match expr {
            Expr::Add(src, dst, sel) => {
                let src = self.visit_expr(src);
                let dst = self.visit_expr(dst);
                // TODO: interpret sel
                self.add_data(src, dst)
            },
            Expr::Int(_) => {
                Operand::new(Loc::Imm(expr), INT, Pos::bof(0))
            }
            _ => {
                panic!("not supported yet");
            }
        }
    }

//    pub fn add(&mut self, expr: &Expr<'a>, src: Operand, dst: Operand, sel: &str) -> Operand {
//        require_types_eq_scalar(&src, &dst);
//        let modes = (src.loc.mode(), dst.loc.mode(), sel);
//        match modes {
//            (M_Imm,  M_Imm,  _  )                      => self.add_const(expr, src, dst),
//            (M_Data, _,      "g") if dst.loc.is(M_Dst) => self.add_data(src, dst),
//            (_,      M_Data, "g") if src.loc.is(M_Src) => self.add_data(src, dst),
//            // ...others...
//            (M_Data, _,      _  ) if dst.loc.is(M_Dst) => self.add_data(src, dst),
//            (_,      M_Data, _  ) if src.loc.is(M_Src) => self.add_data(src, dst),
//            _                                          => dst
//        }
//    }

    pub fn add_data<'a>(&mut self, src: Operand<'a>, dst: Operand<'a>) -> Operand<'a> {
        self.write_ins_s2("add", U8, &src, &dst);
        dst
    }

//    fn add_const(&mut self, expr: &Expr<'a>, src: Operand, dst: Operand) -> Operand {
//        let a   = src.loc.downcast_ref::<Imm>().unwrap();
//        let b   = dst.loc.downcast_ref::<Imm>().unwrap();
//        let loc = match (&a.0, &b.0) {
//            (&Expr::Int(ref a), &Expr::Int(ref b)) => Imm(Expr::Int(a + b)),
//            _                                      => Imm(expr.clone())
//        };
//        Operand::new(loc, INT, src.pos)
//    }

    fn write_ins_s2<A: Display, B: Display>
                   (&mut self, op: &str, ty: &Type, a: &A, b: &B) {
        let suf = Self::size_suffix(ty);
        writeln!(self.out, "    {}{} {}, {}", op, suf, a, b).unwrap();
    }

    fn size_suffix(ty: &Type) -> &'static str {
        match ty.store_width() {
            Some(8)  => ".b",
            Some(16) => ".w",
            Some(32) => ".l",
            _        => ""
        }
    }
}

//fn require_types_eq_scalar<'b>(a: &Operand, b: &'b Operand) -> &'b Type<'b> {
//    match (&*a.ty, &*b.ty) {
//        (&Type::Int(a_), &Type::Int(b_)) => {
//            if a_ == b_ || a_.is_none() { return a.ty }
//            else if        b_.is_none() { return b.ty }
//        },
//        _ => ()
//    }
//    panic!("Type mismatch."); // TODO: Error
//}

#[cfg(test)]
mod tests {
    use num::bigint::ToBigInt;
    use std::io;

    use ast::Expr;
    use mcf5307::loc::*;
    use mcf5307::loc::DataReg::*;
    //use mcf5307::loc::AddrReg::*;
    use super::*;
    use types::*;
    use util::*;

    #[test]
    fn foo() {
        let n   = 4u8.to_bigint().unwrap();
        let src = Operand::new(Loc::Imm(Expr::Int(n)), U8, Pos::bof(0));
        let dst = Operand::new(Loc::Data(D3),          U8, Pos::bof(0));

        let mut gen = CodeGen::new(io::stdout());
        let res = gen.add_data(src, dst.clone());

        assert_eq!(dst, res);
    }
}

