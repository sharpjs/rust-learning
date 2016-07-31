// Type Resolution
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

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry::*;

use aex::ast::{Ast, Stmt, TypeDef};
use aex::scope::{ScopeMap, TypeScope};
use aex::types::*;

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct ResolvedType<'a> {
    pub ast:  &'a Type<'a>,
    pub info: TypeInfo,
}

// -----------------------------------------------------------------------------

pub trait ResolveType<'a> {
    fn resolve(&self, ty: &'a Type<'a>) -> Result<ResolvedType<'a>, ()>;
}

impl<'a> ResolveType<'a> for TypeScope<'a> {
    fn resolve(&self, ty: &'a Type<'a>) -> Result<ResolvedType<'a>, ()> {
        TypeResolver::new(self)
            .resolve(ty)
            .map(|info| ResolvedType { ast: ty, info: info })
    }
}

// -----------------------------------------------------------------------------

/// Defines types as expressed by type definitions in an AST scope.
/// Does not recurse into subscopes.
///
pub fn define_types<'a>(ast:   &'a Ast<'a>,
                        scope: &mut ScopeMap<'a, ResolvedType<'a>>,
                       ) -> Result<(), ()> {
    // Collect type definitions into vec
    let vec = tydefs_to_vec(ast);
    if vec.is_empty() { return Ok(()) }

    // Collect type definitions into map
    let map = try!(tydefs_to_map(&vec));

    // Verify soundness and define the types
    process_typedefs(&vec, &map, scope)
}

fn tydefs_to_vec<'a>(ast: &'a Ast<'a>)
                    -> Vec<&'a TypeDef<'a>> {
    ast
        .iter()
        .filter_map(|s| match *s {
            Stmt::TypeDef(ref tydef) => Some(tydef),
            _                        => None,
        })
        .collect()
}

fn tydefs_to_map<'a>(vec: &Vec<&'a TypeDef<'a>>)
                    -> Result<HashMap<&'a str, &'a TypeDef<'a>>, ()> {
    let mut map = HashMap::new();
    let mut ok  = true;

    for &tydef in vec {
        match map.entry(tydef.id.name) {
            Vacant   (e) => { e.insert(tydef); },
            Occupied (e) => {
                // err: "Duplicate type definition."
                ok = false
            }
        }
    }

    if ok { Ok(map) } else { Err(()) }
}

fn process_typedefs<'a>(vec:   &Vec<&'a TypeDef<'a>>,
                        map:   &HashMap<&'a str, &'a TypeDef<'a>>,
                        scope: &mut ScopeMap<'a, ResolvedType<'a>>,
                       ) -> Result<(), ()>
{
    let ctx = InTypeDef {
        defs:  &map,
        scope: RefCell::new(scope)
    };

    let mut ok = true;

    for &tydef in vec {
        let mut res = TypeResolver::new(&ctx);
        match res.resolve(&tydef.ty) {
            Ok(info) => {
                ok &= ctx.scope
                    .borrow_mut()
                    .define(tydef.id.name, ResolvedType {
                        ast:  &tydef.ty,
                        info: info
                    })
                    .is_ok()
            },
            Err(_) => {
                ok = false
            }
        }
    }

    if ok { Ok(()) } else { Err(()) }
}

// -----------------------------------------------------------------------------

trait Context<'a>: Sized {
    fn lookup<'b>(&self,
                  name: &'a str,
                  res:  &mut TypeResolver<'b, 'a, Self>
                 ) -> Result<TypeInfo, ()>;

    fn err_not_found(&self, name: &str) -> Result<TypeInfo, ()> { Err(()) }
    fn err_duplicate(&self, name: &str) -> Result<TypeInfo, ()> { Err(()) }
    fn err_circular (&self, name: &str) -> Result<TypeInfo, ()> { Err(()) }
}

// -----------------------------------------------------------------------------

impl<'a> Context<'a> for TypeScope<'a> {
    fn lookup<'b>(&self,
                  name: &'a str,
                  res:  &mut TypeResolver<'b, 'a, Self>
                 ) -> Result<TypeInfo, ()> {
        // Look up type; if found it's already resolved
        match self.lookup(name) {
            Some(ty) => Ok(ty.info),
            None     => self.err_not_found(name),
        }
    }
}

// -----------------------------------------------------------------------------

struct InTypeDef<'r, 'a: 'r> {
    defs:  &'r HashMap<&'a str, &'a TypeDef<'a>>,
    scope: RefCell<&'r mut TypeScope<'a>>,
}

impl<'r, 'a: 'r> Context<'a> for InTypeDef<'r, 'a> {
    fn lookup<'b>(&self,
                  name: &'a str,
                  res:  &mut TypeResolver<'b, 'a, Self>
                 ) -> Result<TypeInfo, ()> {

        // Check if type is resolved already
        match self.scope.borrow().lookup(name) {
            Some(ty) => return Ok(ty.info),
            None     => (),
        }

        // Check if type is defined in current scope
        let def = match self.defs.get(name) {
            Some(&def) => def,
            None       => return self.err_not_found(name),
        };

        // Resolve definition
        let info = try!(res.resolve(&def.ty));
        let ty   = ResolvedType { ast: &def.ty, info: info };

        let ty = self.scope.borrow_mut().define(name, ty);
        let ty = self.scope.borrow().lookup(name).unwrap();
        Ok(ty.info)
    }
}

// -----------------------------------------------------------------------------

struct TypeResolver<'r, 'a: 'r, C: 'r + Context<'a>> {
    used: Used<'a>,
    ctx:  &'r C
}

impl<'r, 'a: 'r, C: 'r + Context<'a>> TypeResolver<'r, 'a, C> {
    fn new(ctx: &'r C) -> Self {
        TypeResolver { used: Used::none(), ctx: ctx }
    }

    fn sub(&self) -> Self {
        Self::new(self.ctx)
    }

    fn resolve(&mut self, ty: &Type<'a>) -> Result<TypeInfo, ()> {
        match *ty {
            Type::Ref    (ref r) => self.resolve_ref    (r),
            Type::Int    (ref i) => self.resolve_int    (i),
            Type::Float  (ref f) => self.resolve_float  (f),
            Type::Array  (ref a) => self.resolve_array  (a),
            Type::Ptr    (ref p) => self.resolve_ptr    (p),
            Type::Struct (ref s) => self.resolve_struct (s),
            Type::Union  (ref u) => self.resolve_union  (u),
            Type::Func   (ref f) => self.resolve_func   (f),
        }
    }

    fn resolve_ref(&mut self, r: &TyRef<'a>) -> Result<TypeInfo, ()> {
        self.ctx.lookup(r.id.name, self)
    }

    fn resolve_int(&mut self, i: &IntTy<'a>) -> Result<TypeInfo, ()> {
        let spec = match *i {
            IntTy::Abstract              => None,
            IntTy::Concrete { spec, .. } => Some(spec),
        };

        Ok(TypeInfo {
            form:  TypeForm::Inty(spec),
            step:  0,
            count: 1,
        })
    }

    fn resolve_float(&mut self, f: &FloatTy<'a>) -> Result<TypeInfo, ()> {
        let spec = match *f {
            FloatTy::Abstract              => None,
            FloatTy::Concrete { spec, .. } => Some(spec),
        };

        Ok(TypeInfo {
            form:  TypeForm::Floaty(spec),
            step:  0,
            count: 1,
        })
    }

    fn resolve_array(&mut self, a: &ArrayTy<'a>) -> Result<TypeInfo, ()> {
        let info = try!(self.resolve(&a.ty));

        if info.form.size_bytes() == 0 {
            // Array element does not have known size
            return Err(())
        }

        Ok(TypeInfo { count: a.len, .. info })
    }

    fn resolve_ptr(&mut self, p: &PtrTy<'a>) -> Result<TypeInfo, ()> {
        let info = try!(self      .resolve(&p.ptr_ty));
                   try!(self.sub().resolve(&p.val_ty));

        if let TypeForm::Inty(..) = info.form {
            Ok(info)
        } else {
            // Type not valid as a pointer
            return Err(())
        }
    }

    fn resolve_struct(&mut self, s: &StructTy<'a>) -> Result<TypeInfo, ()> {
        let size = try!(
            self.resolve_members(&s.members[..], sum)
        );
        Ok(TypeInfo {
            form:  TypeForm::Opaque(Some(size)),
            step:  0,
            count: 1,
        })
    }

    fn resolve_union(&mut self, u: &UnionTy<'a>) -> Result<TypeInfo, ()> {
        let size = try!(
            self.resolve_members(&u.members[..], max)
        );
        Ok(TypeInfo {
            form:  TypeForm::Opaque(Some(size)),
            step:  0,
            count: 1,
        })
    }

    fn resolve_func(&mut self, f: &FuncTy<'a>) -> Result<TypeInfo, ()> {
        try!(self.sub_resolve_members(&f.params[..]));
        try!(self.sub_resolve_members(&f.rets  [..]));
        Ok(TypeInfo {
            form:  TypeForm::Opaque(None),
            step:  0,
            count: 1,
        })
    }

    fn resolve_members<F>(&mut self, members: &[Member<'a>], f: F)
                         -> Result<usize, ()>
                         where F: Fn(usize, usize) -> usize {
        let mut size = 0;
        let mut ok   = true;
    
        for member in members {
            let bytes = match self.resolve(&member.ty) {
                Ok  (ref info) => info.form.size_bytes(),
                Err (()      ) => { ok = false; continue }
            };

            if bytes == 0 {
                // err: unsized type
                ok = false; continue
            }

            size = f(size, bytes)
        }
    
        if ok { Ok(size) } else { Err(()) }
    }

    fn sub_resolve_members(&mut self, members: &[Member<'a>])
                          -> Result<(), ()> {
        let mut ok = true;
    
        for member in members {
            if self.sub().resolve(&member.ty).is_err() {
                ok = false
            }
        }

        if ok { Ok(()) } else { Err(()) }
    }
}

#[inline]
fn sum(a: usize, b: usize) -> usize {
    a + b
}

#[inline]
fn max(a: usize, b: usize) -> usize {
    if a > b { a } else { b }
}

// -----------------------------------------------------------------------------

struct Used<'a> {
    used: Option<HashSet<&'a str>>
}

impl<'a> Used<'a> {
    fn none() -> Self {
        Used { used: None }
    }

    fn one(name: &'a str) -> Self {
        let mut h = HashSet::new();
        h.insert(name);
        Used { used: Some(h) }
    }

    fn mark(&mut self, name: &'a str) -> bool {
        if self.used.is_none() {
            self.used = Some(HashSet::new());
        }
        let used = self.used.as_mut().unwrap();
        used.insert(name)
    }
}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
}

