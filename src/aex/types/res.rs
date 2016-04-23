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

use std::rc::Rc;
//use num::{BigInt /*, ToPrimitive*/};

//use aex::types::contains::Contains;
//use aex::types::int::IntSpec;
//use aex::types::float::FloatSpec;
use aex::types::{Type, Member};
////use aex::pos::Source;
use aex::message::Messages;
use aex::util::Lookup;

pub struct TypeResolver<'me, 'a, L>
where 'a: 'me,
      L: 'me + Fn(&str) -> Option<Rc<Type<'a>>> {

    lookup: &'me L,
    log:    &'me mut Messages<'a>,
}

impl<'me, 'a, L> TypeResolver<'me, 'a, L>
where 'a: 'me,
      L: 'me + Fn(&str) -> Option<Rc<Type<'a>>> {

    pub fn new(lookup: &'me L, log: &'me mut Messages<'a>) -> Self {
        TypeResolver { lookup: lookup, log: log }
    }

    pub fn resolve(&self, ty: &mut Type<'a>) -> Result<(), ()> {
        match *ty {
            Type::Ident(_, name, ref mut ty) => {
                match (self.lookup)(name) {
                    t@Some(_) => *ty = t,
                    _         => return Err(()),
                }
            },
            Type::Array(_, ref mut ty, _) => {
                try!(self.resolve(ty));
            },
            Type::Ptr(_, ref mut addr_ty, ref mut data_ty) => {
                try!(self.resolve(addr_ty));
                try!(self.resolve(data_ty));
            },
            Type::Struct(_, ref mut members) => {
                try!(self.resolve_members(members));
            },
            Type::Union(_, ref mut members) => {
                try!(self.resolve_members(members));
            },
            Type::Func(_, ref mut args, ref mut rets) => {
                try!(self.resolve_members(args));
                try!(self.resolve_members(rets));
            },
            _ => {
                // Remaining types are in resolved form already
            }
        };
        Ok(())
    }

    pub fn resolve_members(&self, members: &mut [Member<'a>])
                          -> Result<(), ()> {
        for member in members {
            try!(self.resolve(&mut member.1));
        }
        Ok(())
    }
}

//impl<'a> TypeRes<'a> {
//}
//
//impl<'a> Contains<BigInt> for TypeRes<'a> {
//    #[inline(always)]
//    fn contains(&self, item: &BigInt) -> Option<bool> {
//        self.form.contains(item)
//    }
//}
//
//impl TypeForm {
//    pub fn is_scalar(&self) -> bool {
//        match *self {
//            TypeForm::Inty   (..) => true,
//            TypeForm::Floaty (..) => true,
//            _                     => false
//        }
//    }
//
//    pub fn value_width(&self) -> Option<u8> {
//        match *self {
//            TypeForm::Inty(Some(IntSpec { value_width, .. })) => {
//                Some(value_width)
//            },
//            TypeForm::Floaty(Some(FloatSpec { value_width, .. })) => {
//                Some(value_width)
//            },
//            _ => None
//        }
//    }
//
//    pub fn store_width(&self) -> Option<u8> {
//        match *self {
//            TypeForm::Inty(Some(IntSpec { store_width, .. })) => {
//                Some(store_width)
//            },
//            TypeForm::Floaty(Some(FloatSpec { store_width, .. })) => {
//                Some(store_width)
//            },
//            _ => None
//        }
//    }
//}
//
//impl Contains<BigInt> for TypeForm {
//    #[inline]
//    fn contains(&self, expr: &BigInt) -> Option<bool> {
//        match *self {
//            TypeForm::Inty   (s) => s.contains(expr),
//            TypeForm::Floaty (s) => None, // Don't know for now
//            TypeForm::Opaque     => Some(false)
//        }
//    }
//}

#[cfg(test)]
mod tests {
}

