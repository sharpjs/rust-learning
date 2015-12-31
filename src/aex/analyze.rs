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

use std::collections::VecDeque;

use aex::ast::*;
use aex::message::*;
use aex::pos::*;
use aex::scope::*;
use aex::symbol::*;
use aex::types::*;

pub struct DeclScanner<'a> {
    scope:    &'a mut Scope<'a>,
    messages: &'a mut Messages<'a>,
    todo:     VecDeque<&'a Stmt<'a>>,
}

impl<'a> DeclScanner<'a> {
    pub fn new( scope:    &'a mut Scope   <'a>,
                messages: &'a mut Messages<'a>,
              ) -> Self {
        DeclScanner {
            scope:    scope,
            messages: messages,
            todo:     VecDeque::new(),
        }
    }

    pub fn scan(&mut self, mut stmt: &'a Stmt<'a>) {
        loop {
            match *stmt {
                // Type Declarations
                Stmt::TypeDef
                    (ref pos, ref name, ref ty)
                    => self.declare_type(pos, name, ty),

                // Symbol Declarations
                Stmt::Label
                    (ref pos, ref name)
                    => self.declare_sym(pos, name, INT),
                    // TODO: Use target ptr type
                Stmt::Bss
                    (ref pos, ref name, ref ty)
                    => self.declare_sym(pos, name, ty),
                Stmt::Data
                    (ref pos, ref name, ref ty, _)
                    => self.declare_sym(pos, name, ty),
                Stmt::Alias
                    (ref pos, ref name, ref ty, _)
                    => self.declare_sym(pos, name, ty),
                Stmt::Func
                    (ref pos, ref name, ref ty, _)
                    => self.declare_sym(pos, name, ty),

                // Compound Statements
                Stmt::Block
                    (_, ref stmts)
                    => self.todo.extend(stmts),

                // Other (ignored)
                _ => {}
            }

            match self.todo.pop_front() {
                Some(s) => stmt = s,
                None    => return,
            }
        }
    }

    fn declare_type(&mut self,
                    pos:  &Pos<'a>,
                    name: &'a str,
                    ty:   &'a Type<'a>) {
        let res = self.scope.types.define_ref(name, ty);
        if let Err(_) = res {
            self.messages.err_type_redefined(pos, name);
        }
    }

    fn declare_sym(&mut self,
                   pos:  &Pos<'a>,
                   name: &'a str,
                   ty:   &'a Type<'a>) {
        let sym = Symbol { name: name, ty: ty };
        let res = self.scope.symbols.define(name, sym);
        if let Err(_) = res {
            self.messages.err_sym_redefined(pos, name);
        }
    }
}

