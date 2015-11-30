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

use ast::Stmt;
use scope::*;

pub struct DeclScanner<'a> {
    scope: &'a mut Scope<'a>
}

impl<'a> DeclScanner<'a> {
    pub fn new(scope: &'a mut Scope<'a>) -> Self {
        DeclScanner { scope: scope }
    }

    pub fn scan(&'a mut self, stmts: &'a Vec<Stmt>) {
        for stmt in stmts {
            match *stmt {
                Stmt::Block(..) => {
                    // TODO: recurse into subscope
                },
                Stmt::TypeDef(ref name, ref decl) => { 
                    let name =  &**name;
                    let ty   = (&**decl).clone(); // TODO: Shouldn't need clone

                    if let Err(ty) = self.scope.define_type(name, ty) {
                        panic!("type already defined: {:?}", &ty)
                    }
                },
                _ => {}
            }
        }

        // TODO: Any labels here are pointers to position at EOF
    }
}

