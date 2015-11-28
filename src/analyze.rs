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

use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::rc::Rc;

use ast::Stmt;
use scope::*;
use types::*;
use util::shared::*;

#[derive(Default)]
struct TypeTodo<'a> {
    spec:      Option<&'a Type>,
    needs:     HashSet<SharedStr>,
    needed_by: HashSet<SharedStr>,
}

impl<'a> TypeTodo<'a> {
    fn with_needed_by(needed_by: SharedStr) -> Self {
        TypeTodo {
            needed_by: HashSet::from_iter(Some(needed_by)),
            .. Default::default()
        }
    }

    fn with_needs(spec: &'a Type, needs: Vec<SharedStr>) -> Self {
        TypeTodo {
            spec:  Some(spec),
            needs: HashSet::from_iter(needs),
            .. Default::default()
        }
    }
}

pub struct DeclAnalyzer<'a> {
    type_todos: HashMap<SharedStr, TypeTodo<'a>>,
}

impl<'a> DeclAnalyzer<'a> {
    pub fn new() -> Self {
        DeclAnalyzer {
            type_todos: HashMap::new()
        }
    }

    pub fn analyze_decls(&mut self, stmts: &'a Vec<Stmt>, scope: &mut Scope) {
        // ordering & cycle detection
        // a. enable type to be a "partial" type (rejected: ugly)
        // b. sort decls into dependency order, then iterate <-- not possible w/anonymous types
        // c. track undef'd decls, remove as defined <-- I like this

        let mut labels = vec![];

        for stmt in stmts {
            match stmt {
                &Stmt::Block(..) => {
                    // TODO: recurse into subscope
                },
                &Stmt::TypeDef(ref name, ref spec) => { 
                    let name: SharedStr = name.clone().into();

                    match self.resolve_type(spec, scope) {
                        Ok(ty) => {
                            // Type is fully definable now
                            scope.define_type(name, ty).unwrap()
                        },
                        Err(needs) => {
                            // Type prerequisites are not defined (yet)
                            //self.add_todo_type(name, spec, needs)
                        }
                    }
                },
                &Stmt::Label(ref name) => {
                    // Label applies to next non-label statement
                    labels.push(name.clone())
                },
                //&Stmt::Bss(ref name, ref ty) => {
                //    let ty = resolve_type(ty);
                //    let sym = Symbol {
                //        name: name.clone().into(),
                //        ty:   Rc::new(ty).into()
                //    };
                //    scope.define_symbol(sym).unwrap();
                //},
                _ => {}
            }
        }

        // TODO: Any labels here are pointers to position at EOF
    }

    fn resolve_type(&mut self, spec: &Type, scope: &Scope)
        -> Result<
            SharedType,     // Ok:  The specified type
            Vec<SharedStr>  // Err: Prerequisite types still undefined
        >
    {
        match spec {
            &Type::Ref(ref name) => {
                match scope.lookup_type(&**name) {
                    Some(ty) => Ok(ty.into()),
                    None     => Err(vec![name.clone().into()])
                }
            },
            &Type::Array(ref item_t, length) => {
                let item_t = try!(self.resolve_type(item_t, scope));
                Ok(Rc::new(Type::Array(item_t, length)).into())
            },
            _ => panic!("not implemented yet")
        }
    }

    fn add_todo_type(&mut self,
                     name:  SharedStr,
                     spec:  &'a Type,
                     needs: Vec<SharedStr>) {
        let todos = &mut self.type_todos;

        // Mark each need as needed by this type
        for need in &needs {
            // Already encountered decl? Update its todo item.
            if let Some(todo) = todos.get_mut(need) {
                todo.needed_by.insert(name.clone());
                continue;
            }
            // Did not encounter decl ye†; add new todo item.
            let todo = TypeTodo::with_needed_by(name.clone());
            todos.insert(need.clone(), todo);
        }

        // Make a todo item for this type
        let todo = TypeTodo::with_needs(spec, needs);
        todos.insert(name, todo);
    }

    fn remove_todo_type(&mut self, name: SharedStr, scope: &mut Scope) {
        // Remove todo item for this type
        let todo  = match self.type_todos.remove(&name) {
            Some(todo) => todo,
            None       => return
        };

        // Visit all types needing this one
        for needed_by in &todo.needed_by {
            // That type no longer needs this one
            let spec = {
                let their = &mut self.type_todos.get_mut(needed_by).unwrap();
                their.needs.remove(&name);
                if their.needs.is_empty() { their.spec } else { None }
            };

            // Process that decl if its needs are all met now
            if let Some(spec) = spec {
                let ty = self.resolve_type(spec, scope).unwrap();
                scope.define_type(name.clone() /* wrong name */, ty).unwrap();
            }
        }
    }
}

