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

use aex::ast::*;
use aex::message::*;
use aex::pos::Pos;
use aex::targets::mcf5307::loc::*;
use aex::types::*;

// -----------------------------------------------------------------------------
// Operand - a machine location with its analyzed type and source position

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Operand<'a> {
    pub loc: Loc<'a>,       // Machine location
    pub ty:  &'a Type<'a>,  // Analyzed type
    pub pos: Pos<'a>,       // Source position
}

impl<'a> Operand<'a> {
    pub fn new(loc: Loc<'a>, ty: &'a Type<'a>, pos: Pos<'a>) -> Self
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

pub struct CodeGen<'a, W: io::Write> {
    out:  W,
    msgs: Messages<'a>,
}

impl<'a, W> CodeGen<'a, W> where W: io::Write {
    pub fn new(out: W) -> Self {
        CodeGen { out: out, msgs: Messages::new() }
    }

    // This is all WIP, just idea exploration.

    pub fn visit_expr(&mut self, expr: &Expr<'a>) -> Result<Operand<'a>, ()> {
        match *expr {
            Expr::Add(ref src, ref dst, sel) => {
                let src = try!(self.visit_expr(src));
                let dst = try!(self.visit_expr(dst));
                self.add(&src, &dst, sel.unwrap_or(""))
            },
            Expr::Int(_) => {
                Ok(Operand::new(Loc::Imm(expr.clone()), INT, Pos::bof("f")))
            }
            _ => {
                Err(())
            }
        }
    }

    pub fn add(&mut self, x: &Operand<'a>, y: &Operand<'a>, sel: &str)
                  -> Result<Operand<'a>, ()> {
        match sel {
            "a" => return self.adda(x, y),
            "d" => return self.addd(x, y),
            "i" => return self.addi(x, y),
            "q" => return self.addq(x, y),
            "x" => return self.addx(x, y),
            _   => {}
        }

        let ty = try!(self.check_types(types_eq_scalar, x, y));

        let modes = (x.loc.mode(), y.loc.mode());
        match modes {
            (M_Imm,  M_Imm )                                    => self.addc(        x, y),
            (_,      M_Addr) if x.loc.is(M_Src)                 => self.op2l("adda", x, y),
            (M_Imm,  _     ) if y.loc.is(M_Dst) && x.loc.is_q() => self.op2l("addq", x, y),
            (M_Imm,  M_Data)                                    => self.op2l("addi", x, y),
            (M_Data, _     ) if y.loc.is(M_Dst)                 => self.op2l("add",  x, y),
            (_,      M_Data) if x.loc.is(M_Src)                 => self.op2l("add",  x, y),
            _                                                   => Err(())
        }
    }

    pub fn adda(&mut self, x: &Operand<'a>, y: &Operand<'a>)
                   -> Result<Operand<'a>, ()> {
        let modes = (x.loc.mode(), y.loc.mode());
        match modes {
            (_, M_Addr) if x.loc.is(M_Src) => self.op2l("adda", x, y),
            _                              => Err(())
        }
    }

    pub fn addd(&mut self, x: &Operand<'a>, y: &Operand<'a>)
                   -> Result<Operand<'a>, ()> {
        let modes = (x.loc.mode(), y.loc.mode());
        match modes {
            (M_Data, _     ) if y.loc.is(M_Dst) => self.op2l("add", x, y),
            (_,      M_Data) if x.loc.is(M_Src) => self.op2l("add", x, y),
            _                                   => Err(())
        }
    }

    pub fn addi(&mut self, x: &Operand<'a>, y: &Operand<'a>)
                   -> Result<Operand<'a>, ()> {
        let modes = (x.loc.mode(), y.loc.mode());
        match modes {
            (M_Imm,  M_Data) => self.op2l("addi", x, y),
            _                => Err(())
        }
    }

    pub fn addq(&mut self, x: &Operand<'a>, y: &Operand<'a>)
                   -> Result<Operand<'a>, ()> {
        let modes = (x.loc.mode(), y.loc.mode());
        match modes {
            (M_Imm,  _) if y.loc.is(M_Dst) && x.loc.is_q() => self.op2l("addq", x, y),
            _                                              => Err(())
        }
    }

    pub fn addx(&mut self, x: &Operand<'a>, y: &Operand<'a>)
                   -> Result<Operand<'a>, ()> {
        let modes = (x.loc.mode(), y.loc.mode());
        match modes {
            (M_Data, M_Data) => self.op2l("addx", x, y),
            _                => Err(())
        }
    }

    fn addc(&mut self, x: &Operand<'a>, y: &Operand<'a>)
                   -> Result<Operand<'a>, ()> {
        let args = (
            x.loc.as_expr(), y.loc.as_expr()
        );
        let expr = match args {
            (&Expr::Int(ref x), &Expr::Int(ref y)) => {
                Expr::Int(x + y)
            },
            (x, y) => {
                Expr::Add(Box::new(x.clone()), Box::new(y.clone()), None)
            }
        };
        Ok(Operand::new(Loc::Imm(expr), INT, x.pos))
    }

    fn op2l(&mut self, op: &str, x: &Operand<'a>, y: &Operand<'a>)
               -> Result<Operand<'a>, ()> {
        let t = try!(self.check_types(types_eq_scalar, x, y));
        self.write_ins_s2(op, t, x, y);
        Ok(y.clone())
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

    fn check_types<'b>
                  (&self,
                   f: fn(&'b Type<'a>, &'b Type<'a>) -> Option<&'b Type<'a>>,
                   x: &'b Operand<'a>,
                   y: &'b Operand<'a>)
                  -> Result<&'b Type<'a>, ()> {
        f(&x.ty, &y.ty).ok_or(())
    }
}

fn types_eq_scalar<'a, 'b>
                  (x: &'b Type<'a>, y: &'b Type<'a>)
                  -> Option<&'b Type<'a>> {
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
    use super::*;

    use std::io;
    use num::bigint::ToBigInt;

    use aex::ast::Expr;
    use aex::pos::Pos;
    use aex::targets::mcf5307::loc::*;
    use aex::targets::mcf5307::loc::DataReg::*;
  //use aex::targets::mcf5307::loc::AddrReg::*;
    use aex::types::*;

    #[test]
    fn foo() {
        let n   = 4u8.to_bigint().unwrap();
        let src = Operand::new(Loc::Imm(Expr::Int(n)), U8, Pos::bof("f"));
        let dst = Operand::new(Loc::Data(D3),          U8, Pos::bof("f"));

        let mut gen = CodeGen::new(io::stdout());
        let res = gen.add(&src, &dst, "").unwrap();

        assert_eq!(dst, res);
    }
}

