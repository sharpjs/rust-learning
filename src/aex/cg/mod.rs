// Code Generation
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

//mod eval;
//mod types;

use aex::compiler::Compiler;
//use aex::scope::Scope;

//use self::eval::Eval;

//use aex::analyze::*;
use aex::ast::*;
use aex::output::Output;
use aex::scope::Scope;
use aex::types::res::define_types;
//
//use self::eval::Eval;
//
//pub mod eval;
//pub mod ops;

// -----------------------------------------------------------------------------
// Code Generator

pub fn generate_code<'a>(compiler: &Compiler,
                         ast:      &'a Ast<'a>,
                         scope:    &mut Scope<'a>,
                         out:      &mut Output<'a>
                        ) -> Result<(), ()> {
    // Pass 1: Type declaration
    try!(define_types(ast, &mut scope.types));

    // Pass 2: Evaluation
    for stmt in ast {
    }

    Ok(())
}

//pub struct CodeGenerator<'me, 'str: 'me> {
//    context:   Context<'me, 'str>,
//    evaluator: &'me Eval,
//}
//
//pub struct Context<'me, 'str: 'me> {
//    pub scope: Scope<'me>,
//    pub out:   &'me mut Output<'str>,
//}
//
//impl<'me, 'str> CodeGenerator<'me, 'str> {
//    pub fn new(out:       &'me mut Output <'str>,
//               evaluator: &'me     Eval,
//              ) -> Self {
//        CodeGenerator {
//            evaluator: evaluator,
//            context:   Context {
//                scope: Scope::new(),
//                out:   out,
//            },
//        }
//    }
//
//    pub fn with_parent<'p: 'me>
//                      (parent: &'me mut CodeGenerator<'p, 'str>)
//                      -> Self {
//        CodeGenerator {
//            evaluator: parent.evaluator,
//            context:   Context {
//                scope: Scope::with_parent(&parent.context.scope),
//                out:   parent.context.out,
//            }
//        }
//    }
//
//    pub fn subscope<'sub>
//                   (&'sub mut self)
//                   -> CodeGenerator<'sub, 'str>
//                   where 'me: 'sub {
//        CodeGenerator::with_parent(self)
//    }
//
//    pub fn visit(&mut self, stmts: &'me [Stmt<'str>]) {
//        // Collect declarations first
//        DeclScanner
//            ::new(self.context.out, &mut self.context.scope)
//            .scan(stmts);
//
//        // Then generate code
//        self.visit_stmts(stmts)
//    }
//
//    pub fn visit_stmts(&mut self, stmts: &[Stmt<'str>]) {
//        for stmt in stmts {
//            match *stmt {
//                Stmt::Block(ref pos, ref stmts) => {
//                    self.subscope().visit(stmts)
//                },
//                Stmt::TypeDef(..) => {
//                    // No code to generate
//                },
//                Stmt::Label(_, ref name) => {
//                    self.context.out.asm.write_label(name);
//                },
//                //Stmt::Bss(_, ref name, ref ty) => {
//                //},
//                //Stmt::Data(_, ref name, ref ty, ref expr) => {
//                //},
//                Stmt::Eval(_, ref expr) => {
//                    self.evaluator.eval(expr, &mut self.context);
//                },
//                _ => {}
//            }
//        }
//    }
//}
//
//// -----------------------------------------------------------------------------
//// Tests
//
//#[cfg(test)]
//mod tests {
//    use super::*;
//    use super::eval::Eval;
//
//    use aex::ast::Expr;
//    use aex::output::Output;
//
//    #[test]
//    fn not_sure_yet() {
//        let mut out  = Output::new();
//        let     eval = FakeEvaluator;
//        let     cg   = CodeGenerator::new(&mut out, &eval);
//
//        // ???
//    }
//
//    struct FakeEvaluator;
//
//    impl Eval for FakeEvaluator {
//        fn eval<'ag, 'str>
//               (self: &    Self,
//                expr: &    Expr   <     'str>,
//                ctx:  &mut Context<'ag, 'str>)
//        {}
//    }
//}
//
