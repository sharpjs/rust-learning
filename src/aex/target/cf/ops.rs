// Freescale ColdFire Operators
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

const BYTE: u8 =  8;
const WORD: u8 = 16;
const LONG: u8 = 32;

static ADD: BinaryOperator = BinaryOperator {
    base:         Operator { chars: "+", prec: 5, assoc: Left },
    const_op:     None,
    implicit_op:  None,
    explicit_ops: &[]
};

//static ADDA: AsmOp2 = AsmOp2 {
//    opcodes:        &[(LONG, "adda.l")],
//    default_width:  LONG,
//    check_modes:    check_modes_src_addr,
//    check_types:    check_types_compat,
//    check_forms:    check_forms_inty,
//};
//
//static SUBA: AsmOp2 = AsmOp2 {
//    opcodes:        &[(LONG, "suba.l")],
//    default_width:  LONG,
//    check_modes:    check_modes_src_addr,
//    check_types:    check_types_compat,
//    check_forms:    check_forms_inty,
//};
//
//static MOVEA: AsmOp2 = AsmOp2 {
//    opcodes:        &[(LONG, "movea.l"),
//                      (WORD, "movea.w")],
//    default_width:  LONG,
//    check_modes:    check_modes_src_addr,
//    check_types:    check_types_compat,
//    check_forms:    check_forms_inty,
//};

//// -----------------------------------------------------------------------------
//
//fn check_modes_src_addr(src: Mode, dst: Mode) -> bool {
//    dst == M_Addr && mode_any(src, M_Src)
//}

