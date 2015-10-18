// Lexical Analyzer
//
// This file is part of AEx.
// Copyright (C) 2015 Jeffrey Sharp
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

use std::collections::HashMap;
use std::mem;

use interner::*;
use message::*;
use message::Message::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Token {
    Id   (StrId),   // Identifier
    Str  (StrId),   // Literal: string
    Char (char),    // Literal: character
    Int  (u64),     // Literal: integer
    KwType,         // Keyword: type
    KwStruct,       // Keyword: struct
    KwUnion,        // Keyword: union
    KwIf,           // Keyword: if
    KwElse,         // Keyword: else
    KwLoop,         // Keyword: loop
    KwWhile,        // Keyword: while
    KwBreak,        // Keyword: break
    KwContinue,     // Keyword: continue
    KwReturn,       // Keyword: return
    KwJump,         // Keyword: jump
    Bang,           // !
    Eos,            // End of statement
    Eof,            // End of file
    Error (Message) // Lexical error
}
use self::Token::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pos {
    pub byte:   usize,  // 0-based byte offset
    pub line:   u32,    // 1-based line number
    pub column: u32,    // 1-based column number
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum State {
    Initial, AfterEos, InId,
    AfterZero, InNumDec, InNumHex, InNumOct, InNumBin,
    InChar, AtCharEnd, InStr,
    InEsc, AtEscHex0, AtEscHex1, AtEscUni0, AtEscUni1,
    AtEof
}
use self::State::*;

const STATE_COUNT: usize = 17;

type ActionTable = (
    [u8; 128],      // Map from 7-bit char to handler index
    &'static [(     // Handlers array
        Transition, // - state transition
        Action      // - custom action
    )]
);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Transition {
    Redo(State),        // stay at same char, set state
    Next(State),        // move to next char, set state
    Push(State, State), // move to next char, set state, save state for later restore
    Pop                 // move to next char, restore state
}
use self::Transition::*;

type Action = Option<
    fn(&mut Context, char) -> Option<Token>
>;

pub struct Lexer<I>
where I: Iterator<Item=char>
{
    iter:       I,                      // remaining chars
    ch:         Option<char>,           // char  after previous token
    state:      State,                  // state after previous token
    context:    Context                 // context object give to actions
}

struct Context {
    start:      Pos,                    // position of token start
    current:    Pos,                    // position of current character
    number:     u64,                    // number builder
    buffer:     String,                 // string builder
    strings:    Interner,               // string interner
    keywords:   HashMap<StrId, Token>   // keyword table
}

// TODO: Return position information.

impl<I> Lexer<I>
where I: Iterator<Item=char>
{
    fn new(mut iter: I) -> Self {
        let ch    = iter.next();
        let state = match ch { Some(_) => Initial, None => AtEof };

        let mut strings  = Interner::new();
        let     keywords = intern_keywords(&mut strings);

        let context = Context {
            start:    Pos { byte: 0, line: 1, column: 1 },
            current:  Pos { byte: 0, line: 1, column: 1 },
            buffer:   String::with_capacity(128),
            number:   0,
            strings:  strings,
            keywords: keywords
        };

        Lexer { iter:iter, ch:ch, state:state, context:context }
    }

    pub fn lex(&mut self) -> Token {
        let mut ch    =      self.ch;
        let mut state =      self.state;
        let     iter  = &mut self.iter;
        let     ctx   = &mut self.context;

        println!("\nstate = {:?}", state);

        loop {
            // Lookup handler for this state and char
            let (c, (transition, action))
                = lookup(&STATES[state as usize], ch);

            println!("{:?} {:?} => {:?} {:?}", state, ch, c, transition);

            // Interpret transition
            let consume = match transition {
                Next(s)    => { state = s;                 true  },
                Redo(s)    => { state = s;                 false },
                Push(s, p) => { state = s; self.state = p; true  },
                Pop        => { state =    self.state;     true  }
            };

            // Consume character and get next
            if consume {
                ctx.current.byte   += c.len_utf8();
                ctx.current.column += 1;
                ch = iter.next();
            }

            // Invoke custom action
            if let Some(func) = action {
                if let Some(token) = func(ctx, c) {
                    // Remember state for next call
                    self.ch    = ch;
                    self.state = state;
                    return token;
                }
            }

            // NOTE: Returning an Error token from an action does not
            // automatically move the lexer into the AtEof state.  This is OK,
            // because the consumer should stop on receiving an Error token.
        }
    }
}

#[inline]
fn intern_keywords(i: &mut Interner) -> HashMap<StrId, Token> {
    let mut h = HashMap::new();
    h.insert(i.intern("type"),     KwType    );
    h.insert(i.intern("struct"),   KwStruct  );
    h.insert(i.intern("union"),    KwUnion   );
    h.insert(i.intern("if"),       KwIf      );
    h.insert(i.intern("else"),     KwElse    );
    h.insert(i.intern("loop"),     KwLoop    );
    h.insert(i.intern("while"),    KwWhile   );
    h.insert(i.intern("break"),    KwBreak   );
    h.insert(i.intern("continue"), KwContinue);
    h.insert(i.intern("return"),   KwReturn  );
    h.insert(i.intern("jump"),     KwJump    );
    h
}

#[inline]
fn lookup(entry: &ActionTable, ch: Option<char>) -> (char, (Transition, Action))
{
    // Lookup A: char -> handler number
    let (n, c) = match ch {
        Some(c) => {
            let n = c as usize;
            if n & 0x7F == n {
                (entry.0[n] as usize, c)    // U+007F and below => table lookup
            } else {
                (1, c)                      // U+0080 and above => 'other'
            }
        },
        None => (0, '\0')                   // EOF
    };

    // Lookup B: handler number -> handler
    (c, entry.1[n])
}

// Alias for 'other'; for readability of tables only.
#[allow(non_upper_case_globals)]
const x: u8 = 1;

static STATES: [ActionTable; STATE_COUNT] = [
    // Initial
    ([
        x, x, x, x, x, x, x, x,  x, 2, 3, x, x, 2, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        2, x, 9, x, x, x, x, 8,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        6, 7, 7, 7, 7, 7, 7, 7,  7, 7, x, 4, x, x, x, x, // 01234567 89:;<=>?
        x, 5, 5, 5, 5, 5, 5, 5,  5, 5, 5, 5, 5, 5, 5, 5, // @ABCDEFG HIJKLMNO
        5, 5, 5, 5, 5, 5, 5, 5,  5, 5, 5, x, x, x, x, 5, // PQRSTUVW XYZ[\]^_
        x, 5, 5, 5, 5, 5, 5, 5,  5, 5, 5, 5, 5, 5, 5, 5, // `abcdefg hijklmno
        5, 5, 5, 5, 5, 5, 5, 5,  5, 5, 5, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Transition       Action
        /* 0: eof */ ( Redo(AtEof),     None                ),
        /* 1: ??? */ ( Redo(AtEof),     Some(error_unrec)   ),
        /* 2: \s  */ ( Next(Initial),   None                ),
        /* 3: \n  */ ( Next(AfterEos),  Some(yield_eos_nl)  ),
        /* 4:  ;  */ ( Next(AfterEos),  Some(yield_eos)     ),
        /* 5: id0 */ ( Next(InId),      Some(begin_id)      ),
        /* 6:  0  */ ( Next(AfterZero), Some(begin_num_dig) ),
        /* 7: 1-9 */ ( Next(InNumDec),  Some(begin_num_dig) ),
        /* 8:  '  */ ( Next(InChar),    Some(begin_str)     ),
        /* 9:  "  */ ( Next(InStr),     Some(begin_str)     ),
//      /* n:  !  */ ( Next(Initial),   Some(yield_bang)    ),
    ]),

    // AfterEos - After end of statement
    ([
        x, x, x, x, x, x, x, x,  x, 2, 3, x, x, 2, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        2, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        x, x, x, x, x, x, x, x,  x, x, x, 2, x, x, x, x, // 01234567 89:;<=>?
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Transition      Action
        /* 0: eof */ ( Redo(AtEof),    None          ),
        /* 1: ??? */ ( Redo(Initial),  None          ),
        /* 2: \s; */ ( Next(AfterEos), None          ),
        /* 3: \n  */ ( Next(AfterEos), Some(newline) ),
    ]),

    // InId - In identifier
    ([
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        2, 2, 2, 2, 2, 2, 2, 2,  2, 2, x, x, x, x, x, x, // 01234567 89:;<=>?
        x, 2, 2, 2, 2, 2, 2, 2,  2, 2, 2, 2, 2, 2, 2, 2, // @ABCDEFG HIJKLMNO
        2, 2, 2, 2, 2, 2, 2, 2,  2, 2, 2, x, x, x, x, 2, // PQRSTUVW XYZ[\]^_
        x, 2, 2, 2, 2, 2, 2, 2,  2, 2, 2, 2, 2, 2, 2, 2, // `abcdefg hijklmno
        2, 2, 2, 2, 2, 2, 2, 2,  2, 2, 2, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Transition     Action
        /* 0: eof */ ( Redo(AtEof),   Some(yield_id) ),
        /* 1: ??? */ ( Redo(Initial), Some(yield_id) ),
        /* 2: id  */ ( Next(InId),    Some(accum_id) ),
    ]),

    // AfterZero - after 0 introducing a number literal
    ([
        x, x, x, x, x, x, x, x,  x, 7, 8, x, x, 7, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        7, 8, x, 8, 8, 8, 8, x,  8, 8, 8, 8, 8, 8, 8, 8, //  !"#$%&' ()*+,-./
        2, 2, 2, 2, 2, 2, 2, 2,  2, 2, 8, 8, 8, 8, 8, 8, // 01234567 89:;<=>?
        8, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, 8, 8, 8, 8, 3, // PQRSTUVW XYZ[\]^_
        8, x, 6, x, x, x, x, x,  x, x, x, x, x, x, x, 5, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  4, x, x, 8, 8, 8, 8, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Transition      Action
        /* 0: eof */ ( Redo(AtEof),    Some(yield_num_zero)  ),
        /* 1: ??? */ ( Redo(AtEof),    Some(err_invalid_num) ),
        /* 2: 0-9 */ ( Next(InNumDec), Some(accum_num_dec)   ),
        /* 3:  _  */ ( Next(InNumDec), None                  ),
        /* 4:  x  */ ( Next(InNumHex), None                  ),
        /* 5:  o  */ ( Next(InNumOct), None                  ),
        /* 6:  b  */ ( Next(InNumBin), None                  ),
        /* 7: \s  */ ( Next(Initial),  Some(yield_num_zero)  ),
        /* 8: opr */ ( Redo(Initial),  Some(yield_num_zero)  ),
    ]),

    // InNumDec - in a decimal number
    ([
        x, x, x, x, x, x, x, x,  x, 4, 5, x, x, 4, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        4, 5, x, 5, 5, 5, 5, x,  5, 5, 5, 5, 5, 5, 5, 5, //  !"#$%&' ()*+,-./
        2, 2, 2, 2, 2, 2, 2, 2,  2, 2, 5, 5, 5, 5, 5, 5, // 01234567 89:;<=>?
        5, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, 5, 5, 5, 5, 3, // PQRSTUVW XYZ[\]^_
        5, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, 5, 5, 5, 5, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Transition      Action
        /* 0: eof */ ( Redo(AtEof),    Some(yield_num)       ),
        /* 1: ??? */ ( Redo(AtEof),    Some(err_invalid_num) ),
        /* 2: 0-9 */ ( Next(InNumDec), Some(accum_num_dec)   ),
        /* 3:  _  */ ( Next(InNumDec), None                  ),
        /* 4: \s  */ ( Next(Initial),  Some(yield_num)       ),
        /* 5: opr */ ( Redo(Initial),  Some(yield_num)       ),
    ]),

    // InNumHex - in a hexadecimal number
    ([
        x, x, x, x, x, x, x, x,  x, 6, 7, x, x, 6, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        6, 7, x, 7, 7, 7, 7, x,  7, 7, 7, 7, 7, 7, 7, 7, //  !"#$%&' ()*+,-./
        2, 2, 2, 2, 2, 2, 2, 2,  2, 2, 7, 7, 7, 7, 7, 7, // 01234567 89:;<=>?
        7, 3, 3, 3, 3, 3, 3, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, 7, 7, 7, 7, 5, // PQRSTUVW XYZ[\]^_
        7, 4, 4, 4, 4, 4, 4, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, 7, 7, 7, 7, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Transition      Action
        /* 0: eof */ ( Redo(AtEof),    Some(yield_num)         ),
        /* 1: ??? */ ( Redo(AtEof),    Some(err_invalid_num)   ),
        /* 2: 0-9 */ ( Next(InNumHex), Some(accum_num_hex_dig) ),
        /* 3: A-F */ ( Next(InNumHex), Some(accum_num_hex_uc)  ),
        /* 4: a-f */ ( Next(InNumHex), Some(accum_num_hex_lc)  ),
        /* 5:  _  */ ( Next(InNumHex), None                    ),
        /* 6: \s  */ ( Next(Initial),  Some(yield_num)         ),
        /* 7: opr */ ( Redo(Initial),  Some(yield_num)         ),
    ]),

    // InNumOct - in an octal number
    ([
        x, x, x, x, x, x, x, x,  x, 4, 5, x, x, 4, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        4, 5, x, 5, 5, 5, 5, x,  5, 5, 5, 5, 5, 5, 5, 5, //  !"#$%&' ()*+,-./
        2, 2, 2, 2, 2, 2, 2, 2,  x, x, 5, 5, 5, 5, 5, 5, // 01234567 89:;<=>?
        5, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, 5, 5, 5, 5, 3, // PQRSTUVW XYZ[\]^_
        5, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, 5, 5, 5, 5, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Transition      Action
        /* 0: eof */ ( Redo(AtEof),    Some(yield_num)       ),
        /* 1: ??? */ ( Redo(AtEof),    Some(err_invalid_num) ),
        /* 2: 0-7 */ ( Next(InNumOct), Some(accum_num_oct)   ),
        /* 3:  _  */ ( Next(InNumOct), None                  ),
        /* 6: \s  */ ( Next(Initial),  Some(yield_num)       ),
        /* 4: opr */ ( Redo(Initial),  Some(yield_num)       ),
    ]),

    // InNumBin - in a binary number
    ([
        x, x, x, x, x, x, x, x,  x, 4, 5, x, x, 4, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        4, 5, x, 5, 5, 5, 5, x,  5, 5, 5, 5, 5, 5, 5, 5, //  !"#$%&' ()*+,-./
        2, 2, x, x, x, x, x, x,  x, x, 5, 5, 5, 5, 5, 5, // 01234567 89:;<=>?
        5, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, 5, 5, 5, 5, 3, // PQRSTUVW XYZ[\]^_
        5, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, 5, 5, 5, 5, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Transition      Action
        /* 0: eof */ ( Redo(AtEof),    Some(yield_num)       ),
        /* 1: ??? */ ( Redo(AtEof),    Some(err_invalid_num) ),
        /* 2: 0-1 */ ( Next(InNumBin), Some(accum_num_bin)   ),
        /* 3:  _  */ ( Next(InNumBin), None                  ),
        /* 6: \s  */ ( Next(Initial),  Some(yield_num)       ),
        /* 4: opr */ ( Redo(Initial),  Some(yield_num)       ),
    ]),

    // InChar <( ' )> : in a character literal
    ([
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        x, x, x, x, x, x, x, 3,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // 01234567 89:;<=>?
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, 2, x, x, x, // PQRSTUVW XYZ[\]^_
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Transition              Action
        /* 0: eof */ ( Redo(AtEof),            Some(error_char_unterm) ),
        /* 1: ??? */ ( Next(AtCharEnd),        Some(accum_str)         ),
        /* 2:  \  */ ( Push(InEsc, AtCharEnd), None                    ),
        /* 3:  '  */ ( Redo(AtEof),            Some(error_char_length) ),
    ]),

    // AtCharEnd <( ' char )> : expecting the end of a character literal
    ([
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        x, x, x, x, x, x, x, 2,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // 01234567 89:;<=>?
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Transition     Action
        /* 0: eof */ ( Redo(AtEof),   Some(error_char_unterm) ),
        /* 1: ??? */ ( Redo(AtEof),   Some(error_char_length) ),
        /* 2:  '  */ ( Next(Initial), Some(yield_char)        ),
    ]),

    // InStr <( " )> : in a string literal
    ([
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        x, x, 3, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // 01234567 89:;<=>?
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, 2, x, x, x, // PQRSTUVW XYZ[\]^_
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Transition           Action
        /* 0: eof */ ( Redo(AtEof),         Some(error_str_unterm) ),
        /* 1: ??? */ ( Next(InStr),         Some(accum_str)        ),
        /* 2:  \  */ ( Push(InEsc, InStr),  None                   ),
        /* 3:  "  */ ( Next(Initial),       Some(yield_str)        ),
    ]),

    // InEsc <( ['"] \ )> : after escape characer
    ([
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        x, x, 8, x, x, x, x, 7,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        2, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // 01234567 89:;<=>?
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, 6, x, x, x, // PQRSTUVW XYZ[\]^_
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, 3, x, // `abcdefg hijklmno
        x, x, 4, x, 5,10, x, x,  9, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Transition        Action
        /*  0: eof */ ( Redo(AtEof),     Some(error_esc_unterm)  ),
        /*  1: ??? */ ( Redo(AtEof),     Some(error_esc_invalid) ),
        /*  2:  0  */ ( Pop,             Some(accum_str_nul)     ),
        /*  3:  n  */ ( Pop,             Some(accum_str_lf)      ),
        /*  4:  r  */ ( Pop,             Some(accum_str_cr)      ),
        /*  5:  t  */ ( Pop,             Some(accum_str_tab)     ),
        /*  6:  \  */ ( Pop,             Some(accum_str)         ),
        /*  7:  '  */ ( Pop,             Some(accum_str)         ),
        /*  8:  "  */ ( Pop,             Some(accum_str)         ),
        /*  9:  x  */ ( Next(AtEscHex0), None                    ),
        /* 10:  u  */ ( Next(AtEscUni0), None                    ),
    ]),

    // AtEscHex0 <( ['"] \ x )> : at byte escape digit 0
    ([
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        2, 2, 2, 2, 2, 2, 2, 2,  2, 2, x, x, x, x, x, x, // 01234567 89:;<=>?
        x, 3, 3, 3, 3, 3, 3, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
        x, 4, 4, 4, 4, 4, 4, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Transition       Action
        /* 0: eof */ ( Redo(AtEof),     Some(error_esc_unterm)  ),
        /* 1: ??? */ ( Redo(AtEof),     Some(error_esc_invalid) ),
        /* 2: 0-9 */ ( Next(AtEscHex1), Some(begin_num_dig)     ),
        /* 3: A-F */ ( Next(AtEscHex1), Some(begin_num_hex_uc)  ),
        /* 4: a-f */ ( Next(AtEscHex1), Some(begin_num_hex_lc)  ),
    ]),

    // AtEscHex1 <( ['"] \ x hex )> : at byte escape digit 1
    ([
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        2, 2, 2, 2, 2, 2, 2, 2,  2, 2, x, x, x, x, x, x, // 01234567 89:;<=>?
        x, 3, 3, 3, 3, 3, 3, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
        x, 4, 4, 4, 4, 4, 4, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Transition   Action
        /* 0: eof */ ( Redo(AtEof), Some(error_esc_unterm)     ),
        /* 1: ??? */ ( Redo(AtEof), Some(error_esc_invalid)    ),
        /* 2: 0-9 */ ( Pop,         Some(accum_str_esc_dig)    ),
        /* 3: A-F */ ( Pop,         Some(accum_str_esc_hex_uc) ),
        /* 4: a-f */ ( Pop,         Some(accum_str_esc_hex_lc) ),
    ]),

    // AtEscUni0 <( ['"] \ u )> : in unicode escape, expecting {
    ([
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        x, x, 0, x, x, x, x, 0,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // 01234567 89:;<=>?
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, 2, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Transition       Action
        /* 0: eof */ ( Redo(AtEof),     Some(error_esc_unterm)  ),
        /* 1: ??? */ ( Redo(AtEof),     Some(error_esc_invalid) ),
        /* 2:  {  */ ( Next(AtEscUni1), Some(begin_num)         ),
    ]),

    // AtEscUni1 <( ['"] \ u { )> : in unicode escape, expecting hex digit
    ([
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        x, x, 0, x, x, x, x, 0,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        2, 2, 2, 2, 2, 2, 2, 2,  2, 2, x, x, x, x, x, x, // 01234567 89:;<=>?
        x, 3, 3, 3, 3, 3, 3, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
        x, 4, 4, 4, 4, 4, 4, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, x, x, 5, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Transition       Action
        /* 0: eof */ ( Redo(AtEof),     Some(error_esc_unterm)  ),
        /* 1: ??? */ ( Redo(AtEof),     Some(error_esc_invalid) ),
        /* 2: 0-9 */ ( Next(AtEscUni1), Some(accum_num_hex_dig) ),
        /* 3: A-F */ ( Next(AtEscUni1), Some(accum_num_hex_uc)  ),
        /* 4: a-f */ ( Next(AtEscUni1), Some(accum_num_hex_lc)  ),
        /* 5:  }  */ ( Pop,             Some(accum_str_esc)     ),
    ]),

//  // StateName <( prior chars )> : state description
//  ([
//      x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ .tn..r..
//      x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
//      x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
//      x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // 01234567 89:;<=>?
//      x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
//      x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
//      x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
//      x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
//  ],&[
        //             Transition      Action
//      //             Next      Consume  Action
//      /* 0: eof */ ( Redo(AtEof), None ),
//      /* 1: ??? */ ( Redo(AtEof), None ),
//  ]),

    // AtEof - At end of file
    ([
        0, 0, 0, 0, 0, 0, 0, 0,  0, 0, 0, 0, 0, 0, 0, 0, // ........ .tn..r..
        0, 0, 0, 0, 0, 0, 0, 0,  0, 0, 0, 0, 0, 0, 0, 0, // ........ ........
        0, 0, 0, 0, 0, 0, 0, 0,  0, 0, 0, 0, 0, 0, 0, 0, //  !"#$%&' ()*+,-./
        0, 0, 0, 0, 0, 0, 0, 0,  0, 0, 0, 0, 0, 0, 0, 0, // 01234567 89:;<=>?
        0, 0, 0, 0, 0, 0, 0, 0,  0, 0, 0, 0, 0, 0, 0, 0, // @ABCDEFG HIJKLMNO
        0, 0, 0, 0, 0, 0, 0, 0,  0, 0, 0, 0, 0, 0, 0, 0, // PQRSTUVW XYZ[\]^_
        0, 0, 0, 0, 0, 0, 0, 0,  0, 0, 0, 0, 0, 0, 0, 0, // `abcdefg hijklmno
        0, 0, 0, 0, 0, 0, 0, 0,  0, 0, 0, 0, 0, 0, 0, 0, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Transition   Action
        /* 0: eof */ ( Redo(AtEof), Some(yield_eof) ),
    ]),
];

#[inline]
fn yield_eof(l: &mut Context, c: char) -> Option<Token> {
    Some(Eof)
}

#[inline]
fn yield_eos(l: &mut Context, c: char) -> Option<Token> {
    Some(Eos)
}

#[inline]
fn yield_eos_nl(l: &mut Context, c: char) -> Option<Token> {
    newline(l, c);
    Some(Eos)
}

#[inline]
fn newline(l: &mut Context, c: char) -> Option<Token> {
    l.current.column = 1;
    l.current.line  += 1;
    None
}

// Identifier Actions

#[inline]
fn begin_id(l: &mut Context, c: char) -> Option<Token> {
    begin_str(l, c);
    accum_str(l, c);
    None
}

#[inline]
fn accum_id(l: &mut Context, c: char) -> Option<Token> {
    accum_str(l, c);
    None
}

#[inline]
fn yield_id(l: &mut Context, c: char) -> Option<Token> {
    let id = l.strings.intern(&l.buffer);

    match l.keywords.get(&id) {
        Some(&k) => Some(k),
        None     => Some(Id(id))
    }
}

// Number actions

#[inline]
fn begin_num(l: &mut Context, c: char) -> Option<Token> {
    l.number = 0;
    None
}

#[inline]
fn begin_num_dig(l: &mut Context, c: char) -> Option<Token> {
    l.number = int_from_dig(c) as u64;
    None
}

#[inline]
fn begin_num_hex_uc(l: &mut Context, c: char) -> Option<Token> {
    l.number = int_from_hex_uc(c) as u64;
    None
}

#[inline]
fn begin_num_hex_lc(l: &mut Context, c: char) -> Option<Token> {
    l.number = int_from_hex_lc(c) as u64;
    None
}

#[inline]
fn accum_num_dec(l: &mut Context, c: char) -> Option<Token> {
    accum_num(l, 10, int_from_dig(c))
}

#[inline]
fn accum_num_hex_dig(l: &mut Context, c: char) -> Option<Token> {
    accum_num(l, 16, int_from_dig(c))
}

#[inline]
fn accum_num_hex_uc(l: &mut Context, c: char) -> Option<Token> {
    accum_num(l, 16, int_from_hex_uc(c))
}

#[inline]
fn accum_num_hex_lc(l: &mut Context, c: char) -> Option<Token> {
    accum_num(l, 16, int_from_hex_lc(c))
}

#[inline]
fn accum_num_oct(l: &mut Context, c: char) -> Option<Token> {
    accum_num(l, 8, int_from_dig(c))
}

#[inline]
fn accum_num_bin(l: &mut Context, c: char) -> Option<Token> {
    accum_num(l, 2, int_from_dig(c))
}

#[inline]
fn accum_num(l: &mut Context, base: u8, digit: u8) -> Option<Token> {
    let mut n = l.number;

    n = match n.checked_mul(base as u64) {
        Some(n) => n,
        None    => { return error_num_overflow(); }
    };

    n = match n.checked_add(digit as u64) {
        Some(n) => n,
        None    => { return error_num_overflow(); }
    };

    l.number = n;
    None
}

#[inline]
fn int_from_dig(c: char) -> u8 {
    c as u8 - 0x30 // c - '0'
}

#[inline]
fn int_from_hex_uc(c: char) -> u8 {
    c as u8 - 0x37 // 10 + c - 'A'
}

#[inline]
fn int_from_hex_lc(c: char) -> u8 {
    c as u8 - 0x57 // 10 + c - 'a'
}

#[inline]
fn yield_num_zero(l: &mut Context, c: char) -> Option<Token> {
    Some(Int(0))
}

#[inline]
fn yield_num(l: &mut Context, c: char) -> Option<Token> {
    Some(Int(l.number))
}

// Character/String Actions

#[inline]
fn begin_str(l: &mut Context, c: char) -> Option<Token> {
    l.buffer.clear();
    None
}

#[inline]
fn accum_str(l: &mut Context, c: char) -> Option<Token> {
    l.buffer.push(c);
    None
}

#[inline]
fn accum_str_nul(l: &mut Context, c: char) -> Option<Token> {
    l.buffer.push('\0');
    None
}

#[inline]
fn accum_str_lf(l: &mut Context, c: char) -> Option<Token> {
    l.buffer.push('\n');
    None
}

#[inline]
fn accum_str_cr(l: &mut Context, c: char) -> Option<Token> {
    l.buffer.push('\r');
    None
}

#[inline]
fn accum_str_tab(l: &mut Context, c: char) -> Option<Token> {
    l.buffer.push('\t');
    None
}

#[inline]
fn accum_str_esc(l: &mut Context, c: char) -> Option<Token> {
    let n = l.number as u32;
    if  n > UNICODE_MAX { return error_esc_overflow(); }
    let c = unsafe { mem::transmute(n) };
    l.buffer.push(c);
    None
}

const UNICODE_MAX: u32 = 0x10FFFF;

#[inline]
fn accum_str_esc_dig(l: &mut Context, c: char) -> Option<Token> {
    accum_num_hex_dig (l, c);
    accum_str_esc     (l, c);
    None
}

#[inline]
fn accum_str_esc_hex_uc(l: &mut Context, c: char) -> Option<Token> {
    accum_num_hex_uc (l, c);
    accum_str_esc    (l, c);
    None
}

#[inline]
fn accum_str_esc_hex_lc(l: &mut Context, c: char) -> Option<Token> {
    accum_num_hex_lc (l, c);
    accum_str_esc    (l, c);
    None
}

#[inline]
fn yield_char(l: &mut Context, c: char) -> Option<Token> {
    let c = l.buffer.chars().next().unwrap(); // TODO: better way
    Some(Char(c))
}

#[inline]
fn yield_str(l: &mut Context, c: char) -> Option<Token> {
    Some(Str(l.strings.add(&l.buffer)))
}

// Punctuation Actions

#[inline]
fn yield_bang(l: &mut Context, c: char) -> Option<Token> { Some(Bang) }

// Diagnostic Actions

#[inline]
fn error_unrec(l: &mut Context, c: char) -> Option<Token> {
    Some(Error(Lex_Invalid))
}

#[inline]
fn err_invalid_num(l: &mut Context, c: char) -> Option<Token> {
    Some(Error(Lex_NumInvalid))
}

#[inline]
fn error_num_overflow() -> Option<Token> {
    Some(Error(Lex_NumOverflow))
}

#[inline]
fn error_char_unterm(l: &mut Context, c: char) -> Option<Token> {
    Some(Error(Lex_CharUnterminated))
}

#[inline]
fn error_char_length(l: &mut Context, c: char) -> Option<Token> {
    Some(Error(Lex_CharLength))
}

#[inline]
fn error_esc_overflow() -> Option<Token> {
    Some(Error(Lex_EscOverflow))
}

#[inline]
fn error_str_unterm(l: &mut Context, c: char) -> Option<Token> {
    Some(Error(Lex_StrUnterminated))
}

#[inline]
fn error_esc_unterm(l: &mut Context, c: char) -> Option<Token> {
    Some(Error(Lex_EscUnterminated))
}

#[inline]
fn error_esc_invalid(l: &mut Context, c: char) -> Option<Token> {
    Some(Error(Lex_EscInvalid))
}

#[cfg(test)]
mod tests {
    use std::str::Chars;
    use super::*;
    use super::Token::*;

    #[test]
    fn empty() {
        lex("").yields(Eof);
    }

    #[test]
    fn space() {
        lex( " \r\t" ).yields(Eof);
        lex( " \r\t1").yields(Int(1)).yields(Eof);
        lex("1 \r\t" ).yields(Int(1)).yields(Eof);
        lex("1 \r\t2").yields(Int(1)).yields(Int(2)).yields(Eof);
    }

    #[test]
    fn eos() {
        lex(";"         ).yields(Eos).yields(Eof);
        lex("\n"        ).yields(Eos).yields(Eof);
        lex(";1"        ).yields(Eos).yields(Int(1)).yields(Eof);
        lex("\n1"       ).yields(Eos).yields(Int(1)).yields(Eof);
        lex("; \r\t\n;" ).yields(Eos).yields(Eof);
        lex("\n \r\t\n;").yields(Eos).yields(Eof);
    }

    #[test]
    fn id() {
        lex("a" ).yields_id("a").yields(Eof);
        lex("a ").yields_id("a").yields(Eof);
        lex("abcdefghijklmnopqrstuvwxyz").yields_id("abcdefghijklmnopqrstuvwxyz").yields(Eof);
        lex("ABCDEFGHIJKLMNOPQRSTUVWXYZ").yields_id("ABCDEFGHIJKLMNOPQRSTUVWXYZ").yields(Eof);
        lex("_0123456789")               .yields_id("_0123456789")               .yields(Eof);
    }

    #[test]
    fn num() {
        lex( "123456789").yields(Int( 123456789)).yields(Eof);
        lex("0x01234567").yields(Int(0x01234567)).yields(Eof);
        lex("0x89ABCDEF").yields(Int(0x89ABCDEF)).yields(Eof);
        lex("0x89abcdef").yields(Int(0x89ABCDEF)).yields(Eof);
        lex("0o01234567").yields(Int(0o01234567)).yields(Eof);
        lex(      "0b01").yields(Int(      0b01)).yields(Eof);
        lex(       "012").yields(Int(        12)).yields(Eof);
        lex(         "0").yields(Int(         0)).yields(Eof);
        lex(    "1__2__").yields(Int(        12)).yields(Eof);
        lex("0x__1__2__").yields(Int(      0x12)).yields(Eof);
        lex(       "0__").yields(Int(         0)).yields(Eof);
        lex(     "0b1  ").yields(Int(         1)).yields(Eof);
        lex(      "0b1;").yields(Int(         1)).yields(Eos).yields(Eof);

        lex("0b19z").yields_error();
    }

    #[test]
    fn char() {
        lex("'a'") .yields(Char('a')).yields(Eof);
        lex("'a")  .yields_error();
        lex("''")  .yields_error();
        lex("'aa'").yields_error();
    }

    #[test]
    fn char_escape() {
        lex("'\\0'" ).yields(Char('\0')).yields(Eof);
        lex("'\\n'" ).yields(Char('\n')).yields(Eof);
        lex("'\\r'" ).yields(Char('\r')).yields(Eof);
        lex("'\\t'" ).yields(Char('\t')).yields(Eof);
        lex("'\\\\'").yields(Char('\\')).yields(Eof);
        lex("'\\\''").yields(Char('\'')).yields(Eof);
        lex("'\\\"'").yields(Char('\"')).yields(Eof);
        lex("'\\a'" ).yields_error();
    }

    #[test]
    fn char_escape_hex() {
        lex("'\\x5A'" ).yields(Char('\u{5A}')).yields(Eof);
        lex("'\\xA5'" ).yields(Char('\u{A5}')).yields(Eof); // TODO: Is this a byte?
        lex("'\\x"    ).yields_error();
        lex("'\\x'"   ).yields_error();
        lex("'\\x5"   ).yields_error();
        lex("'\\x5'"  ).yields_error();
        lex("'\\x5Ax'").yields_error();
    }

    #[test]
    fn char_escape_uni() {
        lex("'\\u"       ).yields_error();
        lex("'\\u'"      ).yields_error();
        lex("'\\u{"      ).yields_error();
        lex("'\\u{'"     ).yields_error();
        lex("'\\u{}'"    ).yields(Char('\u{000}')).yields(Eof);
        lex("'\\u{1Fe}'" ).yields(Char('\u{1Fe}')).yields(Eof);
        lex("'\\u{1Fe}x'").yields_error();
    }

    #[test]
    fn keywords() {
        lex("type").yields(KwType).yields(Eof);
    }

    struct LexerHarness<'a>(Lexer<Chars<'a>>);

    fn lex(input: &str) -> LexerHarness {
        LexerHarness(Lexer::new(input.chars()))
    }

    impl<'a> LexerHarness<'a> {
        fn yields(&mut self, token: Token) -> &mut Self {
            assert_eq!(token, self.0.lex());
            self
        }

        fn yields_id(&mut self, name: &str) -> &mut Self {
            let token = self.0.lex();
            match token {
                Id(n) => assert_eq!(name, self.0.context.strings.get(n)),
                _     => panic!("lex() did not yield an identifier.")
            };
            self
        }

        fn yields_error(&mut self) -> &mut Self {
            let token = self.0.lex();
            match token {
                Error(_) => {}, // expected
                _        => panic!("lex() did not yield an error.")
            };
            self
        }
    }
}

