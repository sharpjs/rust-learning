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
use aex::target::*;
use aex::types::res::define_types;
use aex::types::res::ResolveType;

// -----------------------------------------------------------------------------
// Code Generator

pub fn generate_code<'a>(compiler: &Compiler,
                         ast:      &'a Ast<'a>,
                         scope:    &mut Scope<'a>,
                         out:      &mut Output<'a>
                        ) -> Result<(), ()> {
    let mut gen = Generator {
        scope:    Scope::with_parent(scope),
        compiler: compiler,
        target:   COLDFIRE,
        out:      out,
    };
    gen.visit_block(ast)
}

pub struct Generator<'g, 'a: 'g> {
    pub scope:            Scope<'g>,
    pub compiler: &'g     Compiler,
    pub target:   &'g     Target,
    pub out:      &'g mut Output<'a>,
}

type R = Result<(), ()>;

impl<'g, 'a: 'g> Generator<'g, 'a> {
    fn sub<'c>(&'c mut self) -> Generator<'c, 'a> {
        Generator {
            scope:    Scope::with_parent(&self.scope),
            compiler: self.compiler,
            target:   self.target,
            out:      self.out,
        }
    }

    fn visit_block(&mut self, block: &'a Ast<'a>) -> R {
        let mut sub = self.sub();

        try!(define_types(block, &mut sub.scope.types));

        result!(block
            .iter()
            .map(|stmt| sub.visit_stmt(stmt))
            .fold(true, |ok, r| ok & r.is_ok())
        )
    }

    fn visit_stmt(&mut self, stmt: &'a Stmt<'a>) -> R {
        match *stmt {
            Stmt::Block   (ref s) => self.visit_block    (s),
            Stmt::Label   (ref s) => self.visit_label    (s),
            Stmt::DataLoc (ref s) => self.visit_data_dec (s),
            _                     => Ok(())
        }
    }

    fn visit_label(&mut self, label: &'a Label<'a>) -> R {
        self.out.asm.write_label(label.id.name);
        Ok(())
    }

    fn visit_data_dec(&mut self, data: &'a DataLoc<'a>) -> R {
        self.out.asm.write_label(data.id.name);
        let ty = try!(self.scope.types.resolve(&data.ty));
        let sz = ty.info.size_bytes();
        self.out.asm.write_op_2(".lcomm", data.id.name, &sz);
        Ok(())
    }
}

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

