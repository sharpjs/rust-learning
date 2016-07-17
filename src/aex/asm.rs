// Assembly Language Builder
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

use std::fmt::{self, Display, Formatter, Write};
use std::ops::Deref;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Assembly (String);

const INITIAL_CAPACITY: usize = 4096;

const INDENT: &'static str = "    ";

impl Assembly {
    #[inline(always)]
    pub fn new() -> Self {
        Assembly(String::with_capacity(INITIAL_CAPACITY))
    }

    #[inline]
    pub fn write_line(&mut self) {
        writeln!(self, "").unwrap();
    }

    #[inline]
    pub fn write_label(&mut self, name: &str) {
        writeln!(self, "{}:", name).unwrap();
    }

    #[inline]
    pub fn write_op_0(&mut self, op: &str) {
        writeln!(self, "{}{}", INDENT, op).unwrap();
    }

    #[inline]
    pub fn write_op_1<A: ?Sized>
                     (&mut self, op: &str, a: &A)
                     where A: Display {
        writeln!(self, "{}{} {}", INDENT, op, a).unwrap();
    }

    #[inline]
    pub fn write_op_2<A: ?Sized, B: ?Sized>
                     (&mut self, op: &str, a: &A, b: &B)
                     where A: Display, B: Display {
        writeln!(self, "{}{} {}, {}", INDENT, op, a, b).unwrap();
    }

    #[inline]
    pub fn write_op_3<A: ?Sized, B: ?Sized, C: ?Sized>
                     (&mut self, op: &str, a: &A, b: &B, c: &C)
                     where A: Display, B: Display, C: Display {
        writeln!(self, "{}{} {}, {}, {}", INDENT, op, a, b, c).unwrap();
    }

    pub fn write_op_n(&mut self, op: &str, args: &[&Display]) {
        match args.len() {
            0 => self.write_op_0(op),
            1 => self.write_op_1(op, &args[0]),
            2 => self.write_op_2(op, &args[0], &args[1]),
            3 => self.write_op_3(op, &args[0], &args[1], &args[2]),
            _ => {
                write!(self, "{}{} {}", INDENT, op, &args[0]).unwrap();
                for arg in &args[1..] {
                    write!(self, ", {}", arg).unwrap();
                }
                self.write_line()
            }
        }
    }

    pub fn clear(&mut self) {
        self.0.clear();
        self.0.shrink_to_fit();
    }
}

impl Deref for Assembly {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &str { self.0.deref() }
}

impl Write for Assembly {
    #[inline(always)]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_str(s)
    }

    #[inline(always)]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.0.write_char(c)
    }

    #[inline(always)]
    fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
        self.0.write_fmt(args)
    }
}

// -----------------------------------------------------------------------------

use num::BigInt;
//use aex::ast::Expr;
use aex::util::WriteFn;

pub struct AsmFlavor {
    pub write_int: WriteFn<BigInt>,
}

pub static GAS_FLAVOR: AsmFlavor = AsmFlavor {
    write_int: write_int_c
};

pub static VASM_MOT_FLAVOR: AsmFlavor = AsmFlavor {
    write_int: write_int_moto
};

pub fn write_int_c(f: &mut Formatter, n: &BigInt) -> fmt::Result {
    write!(f, "0x{:X}", n)
}

pub fn write_int_intel(f: &mut Formatter, n: &BigInt) -> fmt::Result {
    write!(f, "0{:X}h", n)
}

pub fn write_int_moto(f: &mut Formatter, n: &BigInt) -> fmt::Result {
    write!(f, "${:X}", n)
}

// numbers:     0xFF     $FF     0FFh
// 
// immediate:   #v       $v
// 
// registers:   %d0      d0
// 
// absolute:    (v).w    [v]
// 
// displaced:   (a0,4)   (4a0)  4(a0)   a0@(4)   [a0+4]

