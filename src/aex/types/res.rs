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

use std::marker::PhantomData;

use aex::types::{Type, Member};
use aex::message::Messages;
use aex::util::Lookup;

pub struct TypeResolver<'me, 's: 'me, 'a: 's, L>
where 'a: 'me,
       L: 'me + Fn(&str) -> Option<&'s Type<'s, 'a>> {

    lookup: &'me L,
    log:    &'me mut Messages<'a>,
    _z:     PhantomData<&'s ()>,
}

impl<'me, 's, 'a: 's, L> TypeResolver<'me, 's, 'a, L>
where 'a: 'me,
       L: 'me + Fn(&str) -> Option<&'s Type<'s, 'a>> {

    pub fn new(lookup: &'me L, log: &'me mut Messages<'a>) -> Self {
        TypeResolver { lookup: lookup, log: log, _z: PhantomData }
    }

    pub fn resolve(&self, ty: &mut Type<'s, 'a>) -> Result<(), ()> {
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

    pub fn resolve_members(&self, members: &mut [Member<'s, 'a>])
                          -> Result<(), ()> {
        for member in members {
            try!(self.resolve(&mut member.1));
        }
        Ok(())
    }
}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
}

