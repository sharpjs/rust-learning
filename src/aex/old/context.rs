// Evaluation Context
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

use aex::mem::StringInterner;
use aex::output::Output;
use aex::scope::Scope;
use aex::target::TargetRef;

pub struct Context<'a> {
    pub scope:   Scope<'a>,                 // named values and types
    pub target:  TargetRef<'a>,             // operators, root scope, evaluator, compiler options
    pub strings: &'a StringInterner,        // strings
    pub out:     &'a mut Output<'a>,        // output code, message log
}

//impl<'a> Context<'a> {
//    fn new<'b: 'a>(scope: Scope<'b>, out: &'a mut Output<'b>) -> Self {
//        use std::mem::transmute;
//
//        // SAFETY: Transmuting to a shorter lifetime, which is OK.
//
//        let scope: Scope<'a>          = unsafe { transmute(scope) };
//        let out:   &'a mut Output<'a> = unsafe { transmute(out  ) };
//
//        Context { scope: scope, out: out }
//    }
//}

