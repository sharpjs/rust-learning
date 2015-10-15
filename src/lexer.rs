use interner::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Token {
    Id      (Symbol),
    Int     (u64),
    Bang,               // !
    Eos,                // End of statement
    Eof,                // End of file
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
    AtEof
}
use self::State::*;

const STATE_COUNT: usize = 9;

type ActionTable = (
    [u8; 128],      // Map from 7-bit char to transition index
    &'static [(     // Transitions
        State,      // - next state
        bool,       // - true => consume this char
        Action      // - custom action
    )]
);

type Action = Option< fn(&mut Context, char) -> Option<Token> >;

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
    // errors
}

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
            // errors
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
            let (c, (next_state, consume, action))
                = lookup(&STATES[state as usize], ch);

            print!("{:?} {:?} => {:?} {:?} {:?}\n",
                   state, ch, c, next_state, consume);

            state = next_state;

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
        }
    }
}

#[inline]
fn lookup(entry: &ActionTable, ch: Option<char>) -> (char, (State, bool, Action))
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
        //             Next       Consume  Action
        /* 0: eof */ ( AtEof     , false , None               ),
        /* 1: ??? */ ( AtEof     , false , Some(lex_error)    ),
        /* 2: \s  */ ( Initial   , true  , None               ),
        /* 3: \n  */ ( AfterEos  , true  , Some(yield_eos_nl) ),
        /* 4:  ;  */ ( AfterEos  , true  , Some(yield_eos)    ),
        /* 5: id0 */ ( InId      , true  , Some(begin_id)     ),
        /* 6:  0  */ ( AfterZero , true  , Some(begin_num)    ),
        /* 7: 1-9 */ ( InNumDec  , true  , Some(begin_num)    ),
//      /* n:  !  */ ( Initial   , true  , Some(yield_bang)   ),
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
        //             Next      Consume  Action
        /* 0: eof */ ( AtEof    , false , None          ),
        /* 1: ??? */ ( Initial  , false , None          ),
        /* 2: \s  */ ( AfterEos , true  , None          ),
        /* 3: \n  */ ( AfterEos , true  , Some(newline) ),
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
        //             Next     Consume  Action
        /* 0: eof */ ( AtEof   , false , Some(yield_id) ),
        /* 1: ??? */ ( Initial , false , Some(yield_id) ),
        /* 2: id  */ ( InId    , true  , Some(accum_id) ),
    ]),
    // AfterZero - In a number literal after initial 0
    ([
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        2, 2, 2, 2, 2, 2, 2, 2,  2, 2, x, x, x, x, x, x, // 01234567 89:;<=>?
        x, 7, 7, 7, 7, 7, 7, 7,  7, 7, 7, 7, 7, 7, 7, 7, // @ABCDEFG HIJKLMNO
        7, 7, 7, 7, 7, 7, 7, 7,  7, 7, 7, x, x, x, x, 3, // PQRSTUVW XYZ[\]^_
        x, 7, 6, 7, 7, 7, 7, 7,  7, 7, 7, 7, 7, 7, 7, 5, // `abcdefg hijklmno
        7, 7, 7, 7, 7, 7, 7, 7,  4, 7, 7, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Next      Consume  Action
        /* 0: eof */ ( AtEof    , false , Some(yield_num_zero)  ),
        /* 1: ??? */ ( Initial  , false , Some(yield_num_zero)  ),
        /* 2: 0-9 */ ( InNumDec , false , None                  ),
        /* 3:  _  */ ( InNumDec , true  , None                  ),
        /* 4:  x  */ ( InNumHex , true  , None                  ),
        /* 5:  o  */ ( InNumOct , true  , None                  ),
        /* 6:  b  */ ( InNumBin , true  , None                  ),
        /* 7: inv */ ( AtEof    , false , Some(err_invalid_num) ),
    ]),
    // InNumDec - in a decimal number
    ([
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        2, 2, 2, 2, 2, 2, 2, 2,  2, 2, x, x, x, x, x, x, // 01234567 89:;<=>?
        x, 4, 4, 4, 4, 4, 4, 4,  4, 4, 4, 4, 4, 4, 4, 4, // @ABCDEFG HIJKLMNO
        4, 4, 4, 4, 4, 4, 4, 4,  4, 4, 4, x, x, x, x, 3, // PQRSTUVW XYZ[\]^_
        x, 4, 4, 4, 4, 4, 4, 4,  4, 4, 4, 4, 4, 4, 4, 4, // `abcdefg hijklmno
        4, 4, 4, 4, 4, 4, 4, 4,  4, 4, 4, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Next      Consume  Action
        /* 0: eof */ ( AtEof    , false , Some(yield_num)       ),
        /* 1: ??? */ ( Initial  , false , Some(yield_num)       ),
        /* 2: 0-9 */ ( InNumDec , true  , Some(accum_num_dec)   ),
        /* 3:  _  */ ( InNumDec , true  , None                  ),
        /* 4: inv */ ( AtEof    , false , Some(err_invalid_num) ),
    ]),
    // InNumHex - in a hexadecimal number
    ([
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        2, 2, 2, 2, 2, 2, 2, 2,  2, 2, x, x, x, x, x, x, // 01234567 89:;<=>?
        x, 3, 3, 3, 3, 3, 3, 6,  6, 6, 6, 6, 6, 6, 6, 6, // @ABCDEFG HIJKLMNO
        6, 6, 6, 6, 6, 6, 6, 6,  6, 6, 6, x, x, x, x, 5, // PQRSTUVW XYZ[\]^_
        x, 4, 4, 4, 4, 4, 4, 6,  6, 6, 6, 6, 6, 6, 6, 6, // `abcdefg hijklmno
        6, 6, 6, 6, 6, 6, 6, 6,  6, 6, 6, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Next      Consume  Action
        /* 0: eof */ ( AtEof    , false , Some(yield_num)         ),
        /* 1: ??? */ ( Initial  , false , Some(yield_num)         ),
        /* 2: 0-9 */ ( InNumHex , true  , Some(accum_num_hex_dig) ),
        /* 3: A-F */ ( InNumHex , true  , Some(accum_num_hex_uc)  ),
        /* 4: a-f */ ( InNumHex , true  , Some(accum_num_hex_lc)  ),
        /* 5:  _  */ ( InNumHex , true  , None                    ),
        /* 6: inv */ ( AtEof    , false , Some(err_invalid_num)   ),
    ]),
    // InNumOct - in an octal number
    ([
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        2, 2, 2, 2, 2, 2, 2, 2,  4, 4, x, x, x, x, x, x, // 01234567 89:;<=>?
        x, 4, 4, 4, 4, 4, 4, 4,  4, 4, 4, 4, 4, 4, 4, 4, // @ABCDEFG HIJKLMNO
        4, 4, 4, 4, 4, 4, 4, 4,  4, 4, 4, x, x, x, x, 3, // PQRSTUVW XYZ[\]^_
        x, 4, 4, 4, 4, 4, 4, 4,  4, 4, 4, 4, 4, 4, 4, 4, // `abcdefg hijklmno
        4, 4, 4, 4, 4, 4, 4, 4,  4, 4, 4, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Next      Consume  Action
        /* 0: eof */ ( AtEof    , false , Some(yield_num)       ),
        /* 1: ??? */ ( Initial  , false , Some(yield_num)       ),
        /* 2: 0-7 */ ( InNumOct , true  , Some(accum_num_oct)   ),
        /* 3:  _  */ ( InNumOct , true  , None                  ),
        /* 4: inv */ ( AtEof    , false , Some(err_invalid_num) ),
    ]),
    // InNumBin - in a binary number
    ([
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        2, 2, 4, 4, 4, 4, 4, 4,  4, 4, x, x, x, x, x, x, // 01234567 89:;<=>?
        x, 4, 4, 4, 4, 4, 4, 4,  4, 4, 4, 4, 4, 4, 4, 4, // @ABCDEFG HIJKLMNO
        4, 4, 4, 4, 4, 4, 4, 4,  4, 4, 4, x, x, x, x, 3, // PQRSTUVW XYZ[\]^_
        x, 4, 4, 4, 4, 4, 4, 4,  4, 4, 4, 4, 4, 4, 4, 4, // `abcdefg hijklmno
        4, 4, 4, 4, 4, 4, 4, 4,  4, 4, 4, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             Next      Consume  Action
        /* 0: eof */ ( AtEof    , false , Some(yield_num)       ),
        /* 1: ??? */ ( Initial  , false , Some(yield_num)       ),
        /* 2: 0-1 */ ( InNumBin , true  , Some(accum_num_bin)   ),
        /* 3:  _  */ ( InNumBin , true  , None                  ),
        /* 4: inv */ ( AtEof    , false , Some(err_invalid_num) ),
    ]),
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
        //             Next   Consume  Action
        /* 0: eof */ ( AtEof , false , Some(yield_eof) ),
    ]),
];

fn yield_eof(l: &mut Context, c: char) -> Option<Token> {
    Some(Eof)
}

fn yield_eos(l: &mut Context, c: char) -> Option<Token> {
    Some(Eos)
}

fn yield_eos_nl(l: &mut Context, c: char) -> Option<Token> {
    l.current.column = 1;
    l.current.line  += 1;
    Some(Eos)
}

fn newline(l: &mut Context, c: char) -> Option<Token> {
    l.current.column = 1;
    l.current.line  += 1;
    None
}

fn begin_id(l: &mut Context, c: char) -> Option<Token> {
    l.buffer.clear();
    l.buffer.push(c);
    None
}

fn accum_id(l: &mut Context, c: char) -> Option<Token> {
    l.buffer.push(c);
    None
}

fn yield_id(l: &mut Context, c: char) -> Option<Token> {
    let n = l.strings.intern(&l.buffer);
    Some(Id(n))
}

fn begin_num(l: &mut Context, c: char) -> Option<Token> {
    // First digit is always 0 or decimal 1-9
    l.number = c as u64 - 0x30; // c - '0'
    None
}

fn accum_num_dec(l: &mut Context, c: char) -> Option<Token> {
    l.number = (l.number * 10) + (c as u64 - 0x30); // c - '0'
    None
}

fn accum_num_hex_dig(l: &mut Context, c: char) -> Option<Token> {
    l.number = (l.number << 4) + (c as u64 - 0x30); // c - '0'
    None
}

fn accum_num_hex_uc(l: &mut Context, c: char) -> Option<Token> {
    l.number = (l.number << 4) + (c as u64 - 0x37); // 10 + c - 'A'
    None
}

fn accum_num_hex_lc(l: &mut Context, c: char) -> Option<Token> {
    l.number = (l.number << 4) + (c as u64 - 0x57); // 10 + c - 'a'
    None
}

fn accum_num_oct(l: &mut Context, c: char) -> Option<Token> {
    l.number = (l.number << 3) + (c as u64 - 0x30); // c - '0'
    None
}

fn accum_num_bin(l: &mut Context, c: char) -> Option<Token> {
    l.number = (l.number << 1) + (c as u64 - 0x30); // c - '0'
    None
}

fn yield_num_zero(l: &mut Context, c: char) -> Option<Token> {
    Some(Int(0))
}

fn yield_num(l: &mut Context, c: char) -> Option<Token> {
    Some(Int(l.number))
}

fn yield_bang(l: &mut Context, c: char) -> Option<Token> { Some(Bang) }

fn lex_error(l: &mut Context, c: char) -> Option<Token> {
    None
}

fn err_invalid_num (l: &mut Context, c: char) -> Option<Token> {
    // "Invalid character in numeric literal."
    None
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
        lex(     "0b19z", Eof            , (1, 6), (1,  6), 0); // TODO: error
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

