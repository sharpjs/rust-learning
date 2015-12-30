// Analysis Phases
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

use aex::ast::*;
use aex::message::*;
use aex::pos::*;
use aex::scope::*;
use aex::symbol::Symbol;
use aex::types::*;

pub struct DeclScanner<'a> {
    scope:    &'a mut Scope<'a>,
    messages: &'a mut Messages<'a>,
    todo:     Vec<&'a Stmt<'a>>,
}

impl<'a> DeclScanner<'a> {
    pub fn new( scope:    &'a mut Scope   <'a>,
                messages: &'a mut Messages<'a>,
              ) -> Self {
        DeclScanner {
            scope:    scope,
            messages: messages,
            todo:     vec![]
        }
    }

    pub fn scan(&mut self, mut stmt: &'a Stmt<'a>) {
        let mut todo = vec![];

        loop {
            match *stmt {
                Stmt::Block
                    (ref stmts)
                    => self.scan_block(stmts),
                Stmt::TypeDef
                    (ref name, ref ty)
                    => self.scan_type_def(name, ty),
                Stmt::Label
                    (ref name)
                    => self.scan_label(name),
                _ => {}
            }

            match todo.pop() {
                Some(s) => stmt = s,
                None    => return,
            }
        }
    }

    fn scan_block(&mut self,
                  stmts: &'a Vec<Stmt<'a>>
                 ) {
        let todo = &mut self.todo;
        todo.reserve(stmts.len());
        for s in stmts { todo.push(s) }
    }

    fn scan_type_def(&mut self,
                     name: &'a str,
                     ty:   &'a Type<'a>
                    ) {
        let res = self.scope.types.define_ref(name, ty);
        if let Err(_) = res {
            self.messages.err_type_redefined(Pos::bof("f"), name);
        }
    }

    fn scan_label(&mut self, name: &'a str) {
        let sym = Symbol { name: name, ty: INT }; // TODO: target pointer type
        let res = self.scope.symbols.define(name, sym);
        if let Err(_) = res {
            self.messages.err_sym_redefined(Pos::bof("f"), name);
        }
    }
}

