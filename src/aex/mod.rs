// Aex Root Module
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

#[macro_use]
mod util;

//mod analyze;  // probably obsolete
mod asm;
mod ast;
mod compiler;
mod context;
mod cg;
mod lexer;
mod mem;
mod message;
mod operator;
mod output;
mod pos;
//mod parser;   // TODO
mod scope;
mod source;
mod symbol;
mod target;
mod token;
mod types;
mod value;

pub use aex::compiler::Compiler;
pub use aex::target::*;

