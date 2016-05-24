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

use aex::ast::*;
use aex::compiler::Compiler;
use aex::output::Output;
use aex::scope::Scope;
use aex::types::res::define_types;

// -----------------------------------------------------------------------------
// Code Generator

pub fn generate_code<'a>(compiler: &Compiler,
                         ast:      &'a Ast<'a>,
                         scope:    &mut Scope<'a>,
                         out:      &mut Output<'a>
                        ) -> Result<(), ()> {
    cg_block(compiler, ast, scope, out)
}

pub fn cg_stmt<'a>(compiler: &Compiler,
                   stmt:     &'a Stmt<'a>,
                   scope:    &mut Scope,
                   out:      &mut Output<'a>
                  ) -> Result<(), ()> {
    Err(())
}

pub fn cg_block<'a>(compiler: &Compiler,
                    block:    &'a Ast<'a>,
                    scope:    &mut Scope,
                    out:      &mut Output<'a>
                   ) -> Result<(), ()> {
    let mut scope = Scope::with_parent(scope);

    try!(define_types(block, &mut scope.types));

    result!(block
        .iter()
        .map(|stmt| cg_stmt(compiler, stmt, &mut scope, out))
        .fold(true, |ok, r| ok & r.is_ok())
    )
}

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

// -----------------------------------------------------------------------------
// Tests

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

