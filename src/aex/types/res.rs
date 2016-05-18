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

use std::collections::HashSet;

use aex::ast::{Ast, Stmt};
use aex::scope::ScopeMap;
use aex::types::*;
use aex::types::form::{TypeForm, TypeInfo};

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct ResolvedType<'a> {
    pub ast:  &'a Type<'a>,
    pub info: TypeInfo,
}

impl<'a> ResolvedType<'a> {
    pub fn compute<'r: 'a>(ty:    &'a Type<'a>,
                           scope: &'r ScopeMap<'a, Type<'a>>,
                           log:   &'r mut ()
                          ) -> Result<Self, ()> {
        TypeResolver
            ::new(scope, log)
            .resolve(ty)
            .map(|info| ResolvedType { ast: ty, info: info })
    }
}

// -----------------------------------------------------------------------------

struct TypeResolver<'r, 'a: 'r> {
    used:  Used<'a>,
    scope: &'r ScopeMap<'a, Type<'a>>,
    log:   &'r mut (),
}

impl<'r, 'a: 'r> TypeResolver<'r, 'a> {
    fn new(scope: &'r ScopeMap<'a, Type<'a>>,
           log:   &'r mut (),
          ) -> Self {
        TypeResolver {
            used:  Used::none(),
            scope: scope,
            log:   log,
        }
    }

    fn child<'c>(&'c mut self) -> TypeResolver<'c, 'a> {
        TypeResolver {
            used:  Used::none(),
            scope: self.scope,
            log:   self.log,
        }
    }

    pub fn resolve(&mut self, ty: &Type<'a>) -> Result<TypeInfo, ()> {
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
        // Look up type name
        let ty = match self.scope.lookup(r.id.name) {
            Some(ty) => ty,
            _ => {
                // Type not found
                return Err(())
            }
        };

        // Disallow circular reference
        if !self.used.mark(r.id.name) {
            // Unsupported circular reference
            return Err(())
        }

        self.resolve(ty)
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
        let info = try!(self        .resolve(&p.ptr_ty));
                   try!(self.child().resolve(&p.val_ty));

        // TODO: Target needs to verify pointer type
        if let TypeForm::Inty(..) = info.form {
            Ok(info)
        } else {
            // Type not valid as a pointer
            return Err(())
        }
    }

    fn resolve_struct(&mut self, s: &StructTy<'a>) -> Result<TypeInfo, ()> {
        Err(())
    }

    fn resolve_union(&mut self, u: &UnionTy<'a>) -> Result<TypeInfo, ()> {
        Err(())
    }

    fn resolve_func(&mut self, f: &FuncTy<'a>) -> Result<TypeInfo, ()> {
        Err(())
    }
}

// -----------------------------------------------------------------------------

pub fn process_typedefs<'a>(ast: &'a Ast<'a>,
                            tys: &mut ScopeMap<'a, Type<'a>>,
                           ) -> Result<(), ()> {
    // Collect type definitions
    let tydefs: Vec<_> = ast
        .iter()
        .filter_map(|n| match *n {
            Stmt::TypeDef(ref tydef) => Some(tydef),
            _ => None,
        })
        .collect();

    // Add defined types to scope
    for &tydef in &tydefs {
        if let Err(e) = tys.define_ref(tydef.id.name, &tydef.ty) {
            panic!("Duplicate type definition.")
        }
    }

    // Verify type definitions are sound
    let mut ok = true;
    for &tydef in &tydefs {
        let used = &mut Used::one(tydef.id.name);
        ok &= verify_type(&tydef.ty, tys, used).is_ok();
    }

    // Done
    if ok { Ok(()) } else { Err(()) }
}

fn verify_type<'a>(ty:   &Type<'a>,
                   tys:  &ScopeMap<'a, Type<'a>>,
                   used: &mut Used<'a>
                  ) -> Result<TypeForm, ()> {
    match *ty {
        Type::Ref(ref r) => {
            let ty = match tys.lookup(r.id.name) {
                Some(ty) => ty,
                _ => {
                    // Type not found
                    return Err(())
                }
            };
            if !used.mark(r.id.name) {
                // Unsupported circular reference
                return Err(())
            }
            verify_type(ty, tys, used)
        },
        Type::Array(ref a) => {
            // must be sized type, or a ref to one
            let form = try!(verify_type(&a.ty, tys, used));
            if form.size_bytes() != 0 {
                Ok(form)
            } else {
                // Array elements must have a known size
                return Err(())
            }
        },
        Type::Ptr(ref p) => {
            // ptr_ty must be integral type, or a ref to one
            let form = try!(verify_type(&p.ptr_ty, tys, used));
                       try!(verify_type(&p.val_ty, tys, &mut Used::none()));
            if let TypeForm::Inty(..) = form {
                Ok(form)
            } else {
                // Pointers must have an integral type
                // TODO: Target needs to verify pointers too
                return Err(())
            }
        },
        Type::Struct(ref s) => {
            let size = try!(verify_members(&s.members[..], tys, used));
            Ok(TypeForm::Opaque(size))
        },
        Type::Union(ref u) => {
            let size = try!(verify_members(&u.members[..], tys, used));
            Ok(TypeForm::Opaque(size))
        },
        Type::Func(ref f) => {
            let a    = verify_members(&f.params[..], tys, &mut Used::none());
            let b    = verify_members(&f.rets  [..], tys, &mut Used::none());
            let size = try!(a) + try!(b);
            Ok(TypeForm::Opaque(size))
        },
        _ => {
            // Remaining types are in resolved form already
            Err(()) // TODO: Should be Ok
        }
    }
}

fn verify_members<'a>(members: &[Member<'a>],
                      tys:     &ScopeMap<'a, Type<'a>>,
                      used:    &mut Used<'a>
                     ) -> Result<usize, ()> {
    let mut size = 0;
    let mut ok   = true;

    for member in members {
        match verify_type(&member.ty, tys, used) {
            Ok(form) => size += 1,
            Err(())  => ok    = false,
        }
    }

    if ok { Ok(size) } else { Err(()) }
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

