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
use aex::util::ToWith;

use super::value::*;

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

pub fn write_abs_16(f: &mut Formatter, c: &CfFlavor, e: &Expr)
                   -> fmt::Result {
    write_abs(f, c, e, "w")
}

pub fn write_abs_32(f: &mut Formatter, c: &CfFlavor, e: &Expr)
                   -> fmt::Result {
    write_abs(f, c, e, "l")
}

pub fn write_abs(f: &mut Formatter, c: &CfFlavor, e: &Expr, s: &str)
                 -> fmt::Result {
    write!(f, "({}).{}", e.with(c.base), s)
}

pub fn write_indirect(f: &mut Formatter, c: &CfFlavor, r: &AddrReg)
                      -> fmt::Result {
    write!(f, "({})", r.with(c))
}

pub fn write_pre_dec(f: &mut Formatter, c: &CfFlavor, r: &AddrReg)
                     -> fmt::Result {
    write!(f, "-({})", r.with(c))
}

pub fn write_post_inc(f: &mut Formatter, c: &CfFlavor, r: &AddrReg)
                      -> fmt::Result {
    write!(f, "({})+", r.with(c))
}

pub fn write_displaced(f: &mut Formatter,
                       c: &CfFlavor,
                       v: &AddrDisp)
                       -> fmt::Result {
    write!(f, "({}, {})",
        (&v.base).with(c),
        (&v.disp).with(c.base)
    )
}

pub fn write_indexed(f: &mut Formatter, c: &CfFlavor, v: &AddrDispIdx)
                     -> fmt::Result {
    write!(f, "({}, {}, {}*{})",
        (&v.base) .with(c),
        (&v.disp) .with(c.base),
        (&v.index).with(c),
        (&v.scale).with(c.base),
    )
}

pub fn write_pc_displaced(f: &mut Formatter, c: &CfFlavor, v: &PcDisp)
                          -> fmt::Result {
    write!(f, "(%pc, {})",
        (&v.disp).with(c.base)
    )
}

pub fn write_pc_indexed(f: &mut Formatter, c: &CfFlavor, v: &PcDispIdx)
                        -> fmt::Result {
    write!(f, "(%pc, {}, {}*{})",
        (&v.disp) .with(c.base),
        (&v.index).with(c),
        (&v.scale).with(c.base),
    )
}

