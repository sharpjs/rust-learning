// Trait Equality Example
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

use std::any::Any;

// AsAny - workaround for inability to cast from &T to &Any

trait AsAny {
    fn as_any(&self) -> &Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &Any { self }
}

// AnimalEq - anything that wants to be equatable with an Animal

trait AnimalEq : AsAny {
    fn dynamic_eq(&self, other: &Animal) -> bool;
}

impl<T: Any + PartialEq + Animal> AnimalEq for T {
    fn dynamic_eq(&self, other: &Animal) -> bool {
        match other.as_any().downcast_ref::<T>() {
            Some(a) => self == a,
            None    => false
        }
    }
}

impl<'a> PartialEq for Animal + 'a {
    fn eq(&self, other: &(Animal + 'a)) -> bool {
        self.dynamic_eq(other)
    }
}

// Animals

#[derive(Clone, Eq, PartialEq, Debug)]
struct Dog (&'static str);

#[derive(Clone, Eq, PartialEq, Debug)]
struct Cat (&'static str);

trait Animal : AnimalEq {}
impl Animal for Dog {}
impl Animal for Cat {}

fn main() {
    let d: &Animal = &Dog("Fido");
    let c: &Animal = &Cat("Manx");
    assert!(c == c);
    assert!(c != d);
}

