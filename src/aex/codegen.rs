// Code Generator
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

use aex::analyze::*;
use aex::ast::*;
use aex::output::*;
//use aex::pos::Pos;
//use aex::types::*;
use aex::scope::Scope;

// -----------------------------------------------------------------------------
// Code Generator

pub struct CodeGenerator<'me, 'str: 'me> {
    out:    &'me mut Output <'str>,
    scope:           Scope  <'me >,
    eval:   &'me     Eval,
}

impl<'me, 'str> CodeGenerator<'me, 'str> {
    pub fn new(out:  &'me mut Output <'str>,
               eval: &'me     Eval,
              ) -> Self {
        CodeGenerator {
            out:    out,
            eval:   eval,
            scope:  Scope::new(),
        }
    }

    pub fn with_parent<'p: 'me>
                      (parent: &'me mut CodeGenerator<'p, 'str>)
                      -> Self {
        CodeGenerator {
            out:    parent.out,
            eval:   parent.eval,
            scope:  Scope::with_parent(&parent.scope),
        }
    }

    pub fn subscope<'sub>
                   (&'sub mut self)
                   -> CodeGenerator<'sub, 'str>
                   where 'me: 'sub {
        CodeGenerator::with_parent(self)
    }

    pub fn visit(&mut self, stmts: &'me [Stmt<'str>]) {
        // Collect declarations first
        DeclScanner
            ::new(self.out, &mut self.scope)
            .scan(stmts);

        // Then generate code
        self.visit_stmts(stmts)
    }

    pub fn visit_stmts(&mut self, stmts: &[Stmt<'str>]) {
        for stmt in stmts {
            match *stmt {
                Stmt::Block(ref pos, ref stmts) => {
                    self.subscope().visit(stmts)
                },
                Stmt::TypeDef(..) => {
                    // No code to generate
                },
                Stmt::Label(_, ref name) => {
                    self.out.asm.write_label(name);
                },
                //Stmt::Bss(_, ref name, ref ty) => {
                //},
                //Stmt::Data(_, ref name, ref ty, ref expr) => {
                //},
                Stmt::Eval(_, ref expr) => {
                    self.eval.eval(expr, self.out, &mut self.scope);
                },
                _ => {}
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Evaluator

pub trait Eval {
    fn eval(
        self:  &    Self,
        expr:  &    Expr,
        out:   &mut Output,
        scope: &mut Scope,
    );
}

//pub struct Evaluator<'me, 'str:'me, 'cg:'me, T:'cg> {
//    out:    &'me mut Output<'str>,
//    scope:  &'me mut Scope <'cg>,
//    target: &'me     TargetEvaluator<Operand=T>,
//}
//
//impl<'me, 'str, 'cg, T> Evaluator<'me, 'str, 'cg, T> {
//    pub fn new(out:    &'me mut Output<'str>,
//               scope:  &'me mut Scope <'cg>,
//               target: &'me     TargetEvaluator<Operand=T>)
//              -> Self {
//        Evaluator {
//            out:    out,
//            scope:  scope,
//            target: target,
//        }
//    }
//
//    fn eval(&mut self, expr: &Expr<'str>) -> Result<T, ()> {
//        match *expr {
//            Expr::Add(ref src, ref dst, sel) => {
//                let src = try!(self.eval(src));
//                let dst = try!(self.eval(dst));
//                let sel = sel.unwrap_or("");
//                TargetEvaluator::add(self, &src, &dst, sel)
//            },
//            // Subtract, etc...
//            _ => {
//                Err(())
//            }
//        }
//    }
//}
//
//impl<'me, 'str, 'cg, T> Eval<'str> for Evaluator<'me, 'str, 'cg, T> {
//    #[inline]
//    #[allow(unused_must_use)]
//    fn eval(&mut self, expr: &Expr) {
//        self.eval(expr);
//    }
//}
//
//pub trait TargetEvaluator {
//    type Operand;
//
//    fn add<'a, 'b, 'c>
//          (ctx: &mut Evaluator<'a, 'b, 'c, Self::Operand>,
//           src: &Self::Operand,
//           dst: &Self::Operand,
//           sel: &'c str)
//          -> Result<Self::Operand, ()>;
//}

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
    use super::*;

    use aex::ast::Expr;
    use aex::output::Output;
    use aex::scope::Scope;

    #[test]
    fn not_sure_yet() {
        let mut out  = Output::new();
        let     eval = FakeEvaluator;
        let     cg   = CodeGenerator::new(&mut out, &eval);

        // ???
    }

    struct FakeEvaluator;

    impl Eval for FakeEvaluator {
        fn eval(
            self:  &    Self,
            expr:  &    Expr,
            out:   &mut Output,
            scope: &mut Scope,
        ) {}
    }
}

