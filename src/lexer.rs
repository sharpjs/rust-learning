use std::mem;
use interner::*;

#[derive(Clone, /*Copy,*/ PartialEq, Eq, Debug)]
pub enum Token {
    Id      (Symbol),
    Int     (u64),
    Char    (char),
    Str     (String),
    Bang,                   // !
    Eos,                    // End of statement
    Eof,                    // End of file
    Error   (&'static str)  // Lexical error
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
    InEsc, AtEscHex0, AtEscHex1, //AtEscUni0, AtEscUniN
    AtEof
}
use self::State::*;

const STATE_COUNT: usize = 15;

type ActionTable = (
    [u8; 128],      // Map from 7-bit char to handler index
    &'static [(     // Handlers array
        Transition, // - state transition
        Action      // - custom action
    )]
);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Transition {
    Redo(State),    // stay at same char, . . . . . . .  set state
    Next(State),    // move to next char, . . . . . . .  set state
    Push(State),    // move to next char, save    state, set state
    Pop             // move to next char, restore state
}
use self::Transition::*;

type Action = Option<
    fn(&mut Context, char) -> Option<Token>
>;

pub struct Lexer<I>
where I: Iterator<Item=char>
{
    iter:       I,              // remaining chars
    ch:         Option<char>,   // char  after previous token
    state:      State,          // state after previous token
    context:    Context         // context object give to actions
}

struct Context {
    start:      Pos,            // position of token start
    current:    Pos,            // position of current character
    number:     u64,            // number builder
    buffer:     String,         // string builder
    strings:    Interner        // string interner
}

// TODO: Return position information.
// TODO: Check for numeric overflow.

impl<I> Lexer<I>
where I: Iterator<Item=char>
{
    fn new(mut iter: I) -> Self {
        let ch      = iter.next();
        let state   = match ch { Some(_) => Initial, None => AtEof };
        let context = Context {
            start:   Pos { byte: 0, line: 1, column: 1 },
            current: Pos { byte: 0, line: 1, column: 1 },
            buffer:  String::with_capacity(128),
            number:  0,
            strings: Interner::new()
        };
        Lexer { iter:iter, ch:ch, state:state, context:context }
    }

    fn lex(&mut self) -> Token {
        let mut ch    =      self.ch;
        let mut state =      self.state;
        let     iter  = &mut self.iter;
        let     ctx   = &mut self.context;

        print!("state = {:?}\n", state);

        loop {
            let (c, (transition, action))
                = lookup(&STATES[state as usize], ch);

            print!("{:?} {:?} => {:?} {:?}\n", state, ch, c, transition);

            let consume = match transition {
                Next(s) => {                     state = s; true  },
                Redo(s) => {                     state = s; false },
                Push(s) => { self.state = state; state = s; true  },
                Pop     => { let s = self.state; state = s; true  }
            };

            if consume {
                ctx.current.byte   += c.len_utf8();
                ctx.current.column += 1;
                ch = iter.next();
            }

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
            // because the parser should really just stop on getting an Error
            // token.
        }
    }
}

#[inline]
fn lookup(entry: &ActionTable, ch: Option<char>) -> (char, (Transition, Action))
{
    let (n, c) = match ch {
        Some(c) => {
            let n = c as usize;
            if n & 0x7F == n {
                // U+007F and below => table lookup
                (entry.0[n] as usize, c)
            } else {
                // U+0080 and above => 'other'
                (1, c)
            }
        },
        None => (0, '\0') // EOF
    };
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
        2, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
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
        /* 8:  "  */ ( Next(InStr),     Some(begin_str)     ),
//      /* n:  !  */ ( Next(Initial),   Some(yield_bang)    ),
    ]),

    // AfterEos - After end of statement
    ([
        x, x, x, x, x, x, x, x,  x, 2, 3, x, x, 2, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        2, 4, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // 01234567 89:;<=>?
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Transition      Action
        /* 0: eof */ ( Redo(AtEof),    None          ),
        /* 1: ??? */ ( Redo(Initial),  None          ),
        /* 2: \s  */ ( Next(AfterEos), None          ),
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
        //             Transition      Action
        /* 0: eof */ ( Redo(AtEof),     Some(error_char_unterm) ),
        /* 1: ??? */ ( Next(AtCharEnd), Some(accum_str)         ),
        /* 2:  \  */ ( Push(InEsc),     None                    ),
        /* 3:  '  */ ( Redo(AtEof),     Some(error_char_length) ),
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
        //             Transition     Action
        /* 0: eof */ ( Redo(AtEof),   Some(error_str_unterm) ),
        /* 1: ??? */ ( Next(InStr),   Some(accum_str)        ),
        /* 2:  \  */ ( Push(InEsc),   None                   ),
        /* 3:  "  */ ( Next(Initial), Some(yield_str)        ),
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
    //  /* 10:  u  */ ( Next(AtEscUni0), None                    ),
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

//  // AtEscHex2 <( ['"] \ u )> : at a character literal byte escape, char 1
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
//      //             Transition      Action
//      //             Next           Consume  Action
//      /* 0: eof */ ( Redo(AtEof), Some(error_char_unterm)   ),
//      /* 1: ??? */ ( Redo(AtEof), Some(error_char_overflow) ),
//      /* 2:  '  */ ( Next(AtCharEnd), Some(makes_char_escape)   ),
//  ]),

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
    let n = l.strings.intern(&l.buffer);
    Some(Id(n))
}

// Number actions

#[inline]
fn begin_num_dig(l: &mut Context, c: char) -> Option<Token> {
    l.number = int_from_dig(c);
    None
}

#[inline]
fn begin_num_hex_uc(l: &mut Context, c: char) -> Option<Token> {
    l.number = int_from_hex_uc(c);
    None
}

#[inline]
fn begin_num_hex_lc(l: &mut Context, c: char) -> Option<Token> {
    l.number = int_from_hex_lc(c);
    None
}

#[inline]
fn accum_num_dec(l: &mut Context, c: char) -> Option<Token> {
    l.number = (l.number * 10) + int_from_dig(c);
    None
}

#[inline]
fn accum_num_hex_dig(l: &mut Context, c: char) -> Option<Token> {
    l.number = (l.number << 4) + int_from_dig(c);
    None
}

#[inline]
fn accum_num_hex_uc(l: &mut Context, c: char) -> Option<Token> {
    l.number = (l.number << 4) + int_from_hex_uc(c);
    None
}

#[inline]
fn accum_num_hex_lc(l: &mut Context, c: char) -> Option<Token> {
    l.number = (l.number << 4) + int_from_hex_lc(c);
    None
}

#[inline]
fn accum_num_oct(l: &mut Context, c: char) -> Option<Token> {
    l.number = (l.number << 3) + int_from_dig(c);
    None
}

#[inline]
fn accum_num_bin(l: &mut Context, c: char) -> Option<Token> {
    l.number = (l.number << 1) + int_from_dig(c);
    None
}

#[inline]
fn int_from_dig   (c: char) -> u64 { c as u64 - 0x30 /*      c - '0' */ }

#[inline]
fn int_from_hex_uc(c: char) -> u64 { c as u64 - 0x37 /* 10 + c - 'A' */ }

#[inline]
fn int_from_hex_lc(c: char) -> u64 { c as u64 - 0x57 /* 10 + c - 'a' */ }

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
    // TODO: Ensure no overflow
    let c = unsafe { mem::transmute(l.number as u32) };
    l.buffer.push(c);
    None
}

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
    Some(Str(l.buffer.clone()))
}

// Punctuation Actions

#[inline]
fn yield_bang(l: &mut Context, c: char) -> Option<Token> { Some(Bang) }

// Diagnostic Actions

#[inline]
fn error_unrec(l: &mut Context, c: char) -> Option<Token> {
    Some(Error("Unrecognized character."))
}

#[inline]
fn err_invalid_num(l: &mut Context, c: char) -> Option<Token> {
    Some(Error("Invalid character in numeric literal."))
}

#[inline]
fn error_char_unterm(l: &mut Context, c: char) -> Option<Token> {
    Some(Error("Unterminated character literal."))
}

#[inline]
fn error_char_length(l: &mut Context, c: char) -> Option<Token> {
    Some(Error("Invalid character literal length.  Character literals must contain exactly one character."))
}

#[inline]
fn error_str_unterm(l: &mut Context, c: char) -> Option<Token> {
    Some(Error("Unterminated string literal."))
}

#[inline]
fn error_esc_unterm(l: &mut Context, c: char) -> Option<Token> {
    Some(Error("Incomplete escape sequence."))
}

#[inline]
fn error_esc_invalid(l: &mut Context, c: char) -> Option<Token> {
    Some(Error("Invalid escape sequence."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Token::*;

    #[test]
    fn eof() {
        lex("", Eof, (1, 1), (1, 1), 0);
    }

    #[test]
    fn space() {
        lex(" \t", Eof, (1, 3), (1, 3), 0);
    }

    #[test]
    fn eos() {
        lex("\n", Eos, (1, 1), (2, 1), 0);
        // TODO: Test after-eos skipping
    }

    #[test]
    fn id() {
        lex_id("a"  , "a",   (1, 1), (1, 2), 0);
        lex_id("a;" , "a",   (1, 1), (1, 2), 0);
        lex_id("_a1", "_a1", (1, 1), (1, 4), 0);
    }

    #[test]
    fn num() {
        lex( "123456789", Int( 123456789), (1, 1), (1, 10), 0);
        lex("0x01234567", Int(0x01234567), (1, 1), (1, 11), 0);
        lex("0x89ABCDEF", Int(0x89ABCDEF), (1, 1), (1, 11), 0);
        lex("0x89abcdef", Int(0x89ABCDEF), (1, 1), (1, 11), 0);
        lex("0o01234567", Int(0o01234567), (1, 1), (1, 11), 0);
        lex(      "0b01", Int(      0b01), (1, 1), (1,  5), 0);
        lex(       "012", Int(        12), (1, 1), (1,  4), 0);
        lex(         "0", Int(         0), (1, 1), (1,  2), 0);
        lex(    "1__2__", Int(        12), (1, 1), (1,  7), 0);
        lex("0x__1__2__", Int(      0x12), (1, 1), (1, 11), 0);
        lex(       "0__", Int(         0), (1, 1), (1,  4), 0);
        lex(     "0b1  ", Int(         1), (1, 1), (1,  4), 0);
        lex(     "0b1;;", Int(         1), (1, 1), (1,  4), 0);

        lex_err("0b19z", (1, 6), (1,  6));
    }

    fn lex(s: &str, t: Token, start: (u32, u32), end: (u32, u32), errs: u32)
    {
        let mut lexer   = Lexer::new(s.chars());
        let     token   = lexer.lex();
        let     context = lexer.context;

        assert_eq!(t, token);

        //assert_eq!(context.start,  start)
        //assert_eq!(context.end,    end)
        //assert_eq!(context.errors, errs);
    }

    fn lex_err(s: &str, start: (u32, u32), end: (u32, u32))
    {
        let mut lexer   = Lexer::new(s.chars());
        let     token   = lexer.lex();
        let     context = lexer.context;

        match token {
            Error(_) => {},
            _        => panic!("Did not return an error.")
        }

        //assert_eq!(context.start,  start)
        //assert_eq!(context.end,    end)
        //assert_eq!(context.errors, errs);
    }

    fn lex_id(s: &str, name: &str, start: (u32, u32), end: (u32, u32), errs: u32)
    {
        let mut lexer   = Lexer::new(s.chars());
        let     token   = lexer.lex();
        let     context = lexer.context;

        match token {
            Id(sym) => assert_eq!(name, context.strings.get(sym)),
            _       => panic!("Did not return an identifier.")
        }

        //assert_eq!(context.start,  start)
        //assert_eq!(context.end,    end)
        //assert_eq!(context.errors, errs);
    }
}

