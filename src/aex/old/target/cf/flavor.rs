// Freescale ColdFire Output Flavors
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

use std::fmt::{self, Formatter, Write};

use aex::asm::*; //AsmFlavor;
use aex::ast::Expr;
use aex::util::{DisplayWith, ToWith};

use super::value::*;

pub struct CfFlavor {
    pub base:                &'static AsmFlavor,
    pub fmt_abs_16:        fn(&mut Formatter, &CfFlavor, &Expr       ) -> fmt::Result,
    pub fmt_abs_32:        fn(&mut Formatter, &CfFlavor, &Expr       ) -> fmt::Result,
    pub fmt_addr_ind:      fn(&mut Formatter, &CfFlavor, &AddrReg    ) -> fmt::Result,
    pub fmt_addr_ind_dec:  fn(&mut Formatter, &CfFlavor, &AddrReg    ) -> fmt::Result,
    pub fmt_addr_ind_inc:  fn(&mut Formatter, &CfFlavor, &AddrReg    ) -> fmt::Result,
    pub fmt_addr_disp:     fn(&mut Formatter, &CfFlavor, &AddrDisp   ) -> fmt::Result,
    pub fmt_addr_disp_idx: fn(&mut Formatter, &CfFlavor, &AddrDispIdx) -> fmt::Result,
    pub fmt_pc_disp:       fn(&mut Formatter, &CfFlavor, &PcDisp     ) -> fmt::Result,
    pub fmt_pc_disp_idx:   fn(&mut Formatter, &CfFlavor, &PcDispIdx  ) -> fmt::Result,
    pub fmt_data_regs:     WriteRegsFn<DataReg>,
    pub fmt_addr_regs:     WriteRegsFn<AddrReg>,
}

pub type WriteRegsFn<R> = fn(bits: u8, regs: &[R; 8], join: bool,
                             f: &mut Formatter, c: &CfFlavor)
                             -> Result<bool, fmt::Error>;

pub static CF_GAS_FLAVOR: CfFlavor = CfFlavor {
    base:              &GAS_FLAVOR,
    fmt_abs_16:        fmt_abs_16,
    fmt_abs_32:        fmt_abs_32,
    fmt_addr_ind:      fmt_addr_ind,
    fmt_addr_ind_dec:  fmt_addr_ind_dec,
    fmt_addr_ind_inc:  fmt_addr_ind_inc,
    fmt_addr_disp:     fmt_addr_disp,
    fmt_addr_disp_idx: fmt_addr_disp_idx,
    fmt_pc_disp:       fmt_pc_disp,
    fmt_pc_disp_idx:   fmt_pc_disp_idx,
    fmt_data_regs:     fmt_regs,
    fmt_addr_regs:     fmt_regs,
};

pub static CF_VASM_MOT_FLAVOR: CfFlavor = CfFlavor {
    base:              &VASM_MOT_FLAVOR,
    fmt_abs_16:        fmt_abs_16,
    fmt_abs_32:        fmt_abs_32,
    fmt_addr_ind:      fmt_addr_ind,
    fmt_addr_ind_dec:  fmt_addr_ind_dec,
    fmt_addr_ind_inc:  fmt_addr_ind_inc,
    fmt_addr_disp:     fmt_addr_disp,
    fmt_addr_disp_idx: fmt_addr_disp_idx,
    fmt_pc_disp:       fmt_pc_disp,
    fmt_pc_disp_idx:   fmt_pc_disp_idx,
    fmt_data_regs:     fmt_regs,
    fmt_addr_regs:     fmt_regs,
};

pub fn fmt_abs_16(f: &mut Formatter, c: &CfFlavor, e: &Expr)
                 -> fmt::Result {
    fmt_abs(f, c, e, "w")
}

pub fn fmt_abs_32(f: &mut Formatter, c: &CfFlavor, e: &Expr)
                 -> fmt::Result {
    fmt_abs(f, c, e, "l")
}

pub fn fmt_abs(f: &mut Formatter, c: &CfFlavor, e: &Expr, s: &str)
              -> fmt::Result {
    write!(f, "({}).{}", e.with(c.base), s)
}

pub fn fmt_addr_ind(f: &mut Formatter, c: &CfFlavor, r: &AddrReg)
                   -> fmt::Result {
    write!(f, "({})", r.with(c))
}

pub fn fmt_addr_ind_dec(f: &mut Formatter, c: &CfFlavor, r: &AddrReg)
                       -> fmt::Result {
    write!(f, "-({})", r.with(c))
}

pub fn fmt_addr_ind_inc(f: &mut Formatter, c: &CfFlavor, r: &AddrReg)
                       -> fmt::Result {
    write!(f, "({})+", r.with(c))
}

pub fn fmt_addr_disp(f: &mut Formatter, c: &CfFlavor, v: &AddrDisp)
                    -> fmt::Result {
    write!(f, "({}, {})",
        (&v.base).with(c),
        (&v.disp).with(c.base)
    )
}

pub fn fmt_addr_disp_idx(f: &mut Formatter, c: &CfFlavor, v: &AddrDispIdx)
                        -> fmt::Result {
    write!(f, "({}, {}, {}*{})",
        (&v.base ).with(c),
        (&v.disp ).with(c.base),
        (&v.index).with(c),
        (&v.scale).with(c.base),
    )
}

pub fn fmt_pc_disp(f: &mut Formatter, c: &CfFlavor, v: &PcDisp)
                  -> fmt::Result {
    write!(f, "({}, {})",
        (&PcReg ).with(c),
        (&v.disp).with(c.base),
    )
}

pub fn fmt_pc_disp_idx(f: &mut Formatter, c: &CfFlavor, v: &PcDispIdx)
                      -> fmt::Result {
    write!(f, "({}, {}, {}*{})",
        (&PcReg  ).with(c),
        (&v.disp ).with(c.base),
        (&v.index).with(c),
        (&v.scale).with(c.base),
    )
}

fn fmt_regs<R: DisplayWith<CfFlavor>>
           (bits: u8, regs: &[R; 8], join: bool, f: &mut Formatter, c: &CfFlavor)
           -> Result<bool, fmt::Error> {
    let mut n     = 0;      // register number
    let mut bit   = 1;      // bit for register in bitmask
    let mut start = None;   // register number starting current range
    let mut join  = join;   // whether next call needs a joining char

    loop {
        // Loop for each register r0-r7, then a final time for a fake r8,
        // to ensure that a range rN-r7 is terminated.

        // Check if register n is in the set
        let has = n < 8 && (bits & bit) != 0;

        // Start or end a chunk of the set
        match (has, start) {
            (true, None) => {
                // Start a chunk
                start = Some(n)
            },
            (false, Some(s)) => {
                // End a chunk
                if join {
                    try!(f.write_char('/'))
                }
                try!(regs[s].fmt(f, c));
                if n > s + 1 {
                    try!(f.write_char('-'));
                    try!(regs[n - 1].fmt(f, c));
                }
                start = None;
                join  = true;
            },
            _ => { /*nop*/ }
        }

        // Advance to next register
        if n == 8 { break }
        n += 1;
        bit = bit.wrapping_shl(1);
    }

    Ok(join)
}

