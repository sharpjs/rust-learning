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

    pub fn visit_expr<'a>(&mut self, expr: &'a Expr<'a>) -> Operand<'a> {
        match *expr {
            Expr::Add(ref src, ref dst, sel) => {
                let src = self.visit_expr(src);
                let dst = self.visit_expr(dst);
                self.add(src, dst, sel.unwrap_or(""))
            },
            Expr::Int(_) => {
                Operand::new(Loc::Imm(expr.clone()), INT, Pos::bof(0))
            }
            _ => {
                panic!("not supported yet");
            }
        }
    }

    pub fn add<'a>(&mut self, src: Operand<'a>, dst: Operand<'a>, sel: &str) -> Operand<'a> {
        let ty = check_types_eq_scalar(&src.ty, &dst.ty).unwrap();
        let modes = (src.loc.mode(), dst.loc.mode(), sel);
        match modes {
            (M_Imm,  M_Imm,  _  )                      => self.add_const(src, dst),
            (M_Data, _,      "g") if dst.loc.is(M_Dst) => self.add_data(ty, src, dst),
            (_,      M_Data, "g") if src.loc.is(M_Src) => self.add_data(ty, src, dst),
            // ...others...
            (M_Data, _,      _  ) if dst.loc.is(M_Dst) => self.add_data(ty, src, dst),
            (_,      M_Data, _  ) if src.loc.is(M_Src) => self.add_data(ty, src, dst),
            _                                          => dst
        }
    }

    pub fn add_data<'a>(&mut self,
                        ty:  &'a Type<'a>,
                        src: Operand<'a>,
                        dst: Operand<'a>)
                       -> Operand<'a> {
        self.write_ins_s2("add", ty, &src, &dst);
        dst
    }

    fn add_const<'a>(&mut self, x: Operand<'a>, y: Operand<'a>) -> Operand<'a> {
        let args = (
            x.loc.to_expr(), y.loc.to_expr()
        );
        let expr = match args {
            (Expr::Int(x), Expr::Int(y)) => {
                Expr::Int(x + y)
            },
            (x, y) => {
                Expr::Add(Box::new(x), Box::new(y), None)
            }
        };
        Operand::new(Loc::Imm(expr), INT, x.pos)
    }

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

fn check_types_eq_scalar<'a>
                        (x: &'a Type<'a>,
                         y: &'a Type<'a>)
                        -> Option<&'a Type<'a>>
{
    match (x, y) {
        (&Type::Int(xx), &Type::Int(yy)) => match (xx, yy) {
            _ if xx == yy => Some(x),
            (_, None)     => Some(x),
            (None, _)     => Some(y),
            _             => None
        },
        _ => None
    }
}

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
        let res = gen.add_data(U8, src, dst.clone());

        assert_eq!(dst, res);
    }
}

