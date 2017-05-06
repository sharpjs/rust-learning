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

use std::io::Result;
use aex::io::DecodeRead;
use super::Opcode;

pub fn decode<R: DecodeRead>(opcodes: &[Opcode], r: &mut R) -> Result<Option<usize>> {
    let word = r.read_u16()?;

    for (i, o) in opcodes.iter().enumerate() {

        // Word must match discriminant bits of opcode
        if word & o.mask.0 != o.bits.0 { continue; }

        // Word must match valid operand set
        if !o.args.decode(r) { continue; }

        // Use this opcode
        return Ok(Some(i))
    }

    Ok(None)
}

// For    assembly : name -> [opcode] -- use  first opcode that matches
// For disassembly : [(opcode, name)] -- find first opcode that matches, then get name
//
//   "movea.l" -> [a]       -- a    is  the opcode  for movea.l
//   "move.l"  -> [b, c]    -- b, c are the opcodes for move.l
//
// for 0x2...:
//   [ (a, "movea.l")
//   , (b, "move.l" )
//   , (c, "move.l" )
//   ]

// Trie in structure, or ad-hoc trie in code (bunch o'matches)?
//   - code: does not require mask in opcodes table any more
//   - code: uses I cache instead of D cache
//   - data: automatic generation
//   - data: easier evolution
//
// First level disasm trie:
//
// 0 -> ori|btst|bchg|bset|andi|subi|addi|eori|cmpi
// 1 -> move.b
// 2 -> move.l|movea.l
// 3 -> move.w|movea.w
// 4 -> (everything else!)
// 5 -> addq|scc|subq|tpf
// 6 -> bra|bsr|bcc
// 7 -> moveq
// 8 -> or|divu.w|divs.w
// 9 -> sub|subx|suba
// A -> (mac stuff)
// B -> cmp|cmpa|eor
// C -> and|mulu.w|muls.wA
// D -> add|addx|adda
// E -> asl|asr|lksl|lsr
// F -> cpushl|wddata|wdebug|(fp stuff)

