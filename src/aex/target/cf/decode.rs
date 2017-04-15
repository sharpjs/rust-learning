// ColdFire Decoding
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

use std::io::{self, BufRead};
use super::{DecodeContext, OPCODES, Operand, Operands};

//use super::Mnemonic::*;
use super::Operands::*;
//use super::Operand::*;
//use super::Size::*;

pub fn decode<R: BufRead>
    (c: &mut DecodeContext<R>)
    -> io::Result<Option<usize>>
{
    let word = c.next()?;

    for (i, o) in OPCODES.iter().enumerate() {

        // Word must match discriminant bits of opcode
        if word & o.mask.0 != o.bits.0 { continue; }

        // Word must match valid operand set
        if !match_operands(word, o.args, c) { continue; }

        // Use this opcode
        return Ok(Some(i))
    }

    Ok(None)
}

fn match_operands<R: BufRead>
    (word: u16, spec: Operands, c: &mut DecodeContext<R>)
    -> bool
{
    match spec {
        Nullary    => true,
        Unary(p)   => match_operand(word, p[0].0, p[0].1, c),
        Binary(p)  => match_operand(word, p[0].0, p[0].1, c)
                   && match_operand(word, p[1].0, p[1].1, c),
        Ternary(p) => match_operand(word, p[0].0, p[0].1, c)
                   && match_operand(word, p[1].0, p[1].1, c)
                   && match_operand(word, p[2].0, p[2].1, c),
    }
}

fn match_operand<R: BufRead>
    (word: u16, spec: Operand, pos: u8, c: &mut DecodeContext<R>)
    -> bool
{
    // TODO
    true
}

