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

//mod asm;
//mod ast;
mod compiler;
//mod codegen;
mod mem;
//mod lexer;
//mod operator;
//mod output;
mod pos;
//mod parser;
//mod scope;
//mod source;
//mod symbol;
//mod target;
mod types;
mod util;
//mod value;

pub use aex::compiler::compile;
//pub use aex::target::ColdFire;

// Not recently visited

//mod message;

//mod analyze;

