// Test Target
//
// This file is part of AEx.
// Copyright (C) 2017 Jeffrey Sharp
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

use std::fmt::{self, Debug, Display, Formatter};
use aex::operator::OperatorTable;
use aex::scope::Scope;
use aex::target::Target;

pub struct TestTarget (BaseTarget);

impl TestTarget {
    pub fn new() -> Self {
        let mut target = TestTarget(BaseTarget::new());
        target.init();
        target
    }

    fn init(&mut self) {
    }
}

impl Target for TestTarget {
    // ...
}

impl<'a> Display for TestTarget {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("test")
    }
}

impl<'a> Debug for TestTarget {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("TestTarget")
    }
}

pub type TestValue = ();

// -----------------------------------------------------------------------------

pub struct BaseTarget {
    pub operators:  OperatorTable,
    pub root_scope: Scope<'static>,
}

impl BaseTarget {
    pub fn new() -> Self {
        BaseTarget {
            operators:  OperatorTable::new(),
            root_scope: Scope::new(),
        }
    }
}

