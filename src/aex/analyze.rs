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

pub struct DeclScanner<'me, 'a: 'me> {
    scope:    &'me mut Scope   <'a>,
    messages: &'me mut Messages<'a>,
    labels:   VecDeque<(&'me Pos<'a>, &'a str)>,
}

impl<'me, 'a> DeclScanner<'me, 'a> {
    pub fn new(scope:    &'me mut Scope   <'a>,
               messages: &'me mut Messages<'a>)
              -> Self {
        DeclScanner {
            scope:    scope,
            messages: messages,
            labels:   VecDeque::new(),
        }
    }

    pub fn scan(&mut self, stmts: &'a [Stmt<'a>]) {
        for stmt in stmts {
            match *stmt {
                // Compound Statements
                Stmt::Block
                    (_, ref stmts)
                    => { self.scan(stmts); self.declare_sym_labels(INT) }

                // Type Declarations
                Stmt::TypeDef
                    (ref pos, ref name, ref ty)
                    => self.declare_type(pos, name, ty),

                // Symbol Declarations
                Stmt::Label
                    (ref pos, ref name)
                    => self.labels.push_back((pos, *name)),
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

                // Executables
                Stmt::Eval  (..) |
                Stmt::Loop  (..) |
                Stmt::If    (..) |
                Stmt::While (..)
                    => self.declare_sym_labels(INT),
            }
        }
    }

    fn declare_type(&mut self,
                    pos:  &   Pos<'a>,
                    name: &'a str,
                    ty:   &'a Type<'a>) {

        let res = self.scope.types.define_ref(name, ty);

        if let Err(_) = res {
            self.messages.err_type_redefined(pos, name);
        }
    }

    fn declare_sym(&mut self,
                   pos:  &   Pos<'a>,
                   name: &'a str,
                   ty:   &'a Type<'a>) {

        self.declare_sym_labels(ty);
        self.declare_sym_named(pos, name, ty);
    }

    fn declare_sym_labels(&mut self,
                          ty: &'a Type<'a>) {

        if self.labels.is_empty() { return }

        // Copy to new vec to avoid borrowck error
        let labels = self.labels.split_off(0);

        for (pos, name) in labels {
            self.declare_sym_named(pos, name, ty);
        }
    }

    fn declare_sym_named(&mut self,
                         pos:  &   Pos<'a>,
                         name: &'a str,
                         ty:   &'a Type<'a>) {

        let sym = Symbol { name: name, ty: ty };
        let res = self.scope.symbols.define(name, sym);

        if let Err(_) = res {
            self.messages.err_sym_redefined(pos, name);
        }
    }
}

