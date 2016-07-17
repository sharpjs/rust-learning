// Freescale ColdFire Target
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

mod value;
//mod code_gen;

use aex::target::Target;

#[derive(Debug)]
pub struct ColdFire;

impl Target for ColdFire {
    // ...
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CfValue {
    DataReg,
    AddrReg,
}

// -------------------------------------------------------------------------

use std::fmt::{self, Formatter};
use aex::asm::AsmFlavor;
use aex::ast::Expr;
use aex::util::{DisplayWith, WriteFn};

pub struct CfFlavor {
    pub base:         AsmFlavor,
    pub write_abs_16: WriteFn<u64>,
}

pub fn write_abs_16<'a>(f: &mut Formatter, c: &CfFlavor, v: &Expr<'a>)
                        -> fmt::Result {
    write_abs(f, c, v, "w")
}

pub fn write_abs_32<'a>(f: &mut Formatter, c: &CfFlavor, v: &Expr<'a>)
                        -> fmt::Result {
    write_abs(f, c, v, "l")
}

#[inline]
pub fn write_abs<'a>(f: &mut Formatter, c: &CfFlavor, v: &Expr<'a>, s: &str)
                     -> fmt::Result {
    try!(f.write_str("("));
    try!(v.fmt(f, &c.base));
    try!(f.write_str(")."));
    try!(f.write_str(s));
    Ok(())
}

