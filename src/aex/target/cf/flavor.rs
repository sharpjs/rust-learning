// Freescale ColdFire Output Flavors
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

use std::fmt::{self, Formatter};

use aex::asm::*; //AsmFlavor;
use aex::ast::Expr;
use aex::util::DisplayWith;

pub struct CfFlavor {
    pub base:         &'static AsmFlavor,
    pub write_abs_16: fn(&mut Formatter, &CfFlavor, &Expr) -> fmt::Result,
    pub write_abs_32: fn(&mut Formatter, &CfFlavor, &Expr) -> fmt::Result,
}

pub static CF_GAS_FLAVOR: CfFlavor = CfFlavor {
    base:         &GAS_FLAVOR,
    write_abs_16: write_abs_16,
    write_abs_32: write_abs_32,
};

pub static CF_VASM_MOT_FLAVOR: CfFlavor = CfFlavor {
    base:         &VASM_MOT_FLAVOR,
    write_abs_16: write_abs_16,
    write_abs_32: write_abs_32,
};

pub fn write_abs_16(f: &mut Formatter, c: &CfFlavor, v: &Expr)
                    -> fmt::Result {
    write_abs(f, c, v, "w")
}

pub fn write_abs_32(f: &mut Formatter, c: &CfFlavor, v: &Expr)
                    -> fmt::Result {
    write_abs(f, c, v, "l")
}

pub fn write_abs(f: &mut Formatter, c: &CfFlavor, v: &Expr, s: &str)
                 -> fmt::Result {
    try!(f.write_str("("));
    try!(v.fmt(f, c.base));
    try!(f.write_str(")."));
    try!(f.write_str(s));
    Ok(())
}

//pub fn write_disp<R: DisplayWith<CfFlavor>>
//                 (f: &mut Formatter, c: &CfFlavor, base: &R, disp: &Expr)
//                 -> fmt::Result {
//    write!(f, "({}, {})", base, disp)
//}

