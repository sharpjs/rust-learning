use symbol::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pos {
    pub byte:   usize,  // 0-based byte offset
    pub line:   u32,    // 1-based line number
    pub column: u32,    // 1-based column number
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Token {
    Id      (Symbol),
    Int     (u32),
    Bang,               // !
    Eos,                // End of statement
    Eof,                // End of file
}
use self::Token::*;

pub trait Lookahead {
    type Item;
    fn current (&    self) -> Self::Item;
    fn next    (&mut self) -> Self::Item;
}

// Begin new idea 2015-10-13

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum State {
    Initial,
    AtEof
}
const STATE_COUNT: usize = 2;
use self::State::*;

type StateEntry = (
    [u8; 128],      // Map from 7-bit char to transition index
    &'static [(     // Transitions
        State,      // - next state
        bool,       // - true => consume this char
        Action      // - custom action
    )]
);

type Action = Option< fn(&mut Context, char) -> Option<Token> >;

static STATES: [StateEntry; STATE_COUNT] = [
    // Initial
    ([
        1, 1, 1, 1, 1, 1, 1, 1,  1, 2, 3, 1, 1, 2, 1, 1, // ........ .tn..r..
        1, 1, 1, 1, 1, 1, 1, 1,  1, 1, 1, 1, 1, 1, 1, 1, // ........ ........
        2, 4, 1, 1, 1, 1, 1, 1,  1, 1, 1, 1, 1, 1, 1, 1, //  !"#$%&' ()*+,-./
        1, 1, 1, 1, 1, 1, 1, 1,  1, 1, 1, 1, 1, 1, 1, 1, // 01234567 89:;<=>?
        1, 1, 1, 1, 1, 1, 1, 1,  1, 1, 1, 1, 1, 1, 1, 1, // @ABCDEFG HIJKLMNO
        1, 1, 1, 1, 1, 1, 1, 1,  1, 1, 1, 1, 1, 1, 1, 1, // PQRSTUVW XYZ[\]^_
        1, 1, 1, 1, 1, 1, 1, 1,  1, 1, 1, 1, 1, 1, 1, 1, // `abcdefg hijklmno
        1, 1, 1, 1, 1, 1, 1, 1,  1, 1, 1, 1, 1, 1, 1, 1, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        /* 0: eof */ ( AtEof   , false , Some(eof)        ),
        /* 1: ??? */ ( AtEof   , true  , Some(lex_error)  ),
        /* 2: \s  */ ( Initial , true  , None             ),
        /* 3: \n  */ ( Initial , true  , Some(newline)    ),
        /* 4:  !  */ ( Initial , true  , Some(yield_bang) ),
    ]),
    // AtEof
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
        /* 0: eof */ ( AtEof , false , Some(eof) ),
    ]),
];

pub struct Lexer2<I>
where I: Iterator<Item=char>
{
    iter:       I,             // remaining chars
    ch:         Option<char>,  // char  after previous token
    state:      State,         // state after previous token
    context:    Context        // context object give to actions
}

struct Context {
    start:      Pos,           // position of token start
    current:    Pos,           // position of current character
    buffer:     String,        // shared string builder
    // interner
    // errors
}

fn lex_error(x: &mut Context, c: char) -> Option<Token> {
    None
}

fn eof(x: &mut Context, c: char) -> Option<Token> {
    Some(Eof)
}

fn newline(x: &mut Context, c: char) -> Option<Token> {
    x.current.column = 1;
    x.current.line  += 1;
    None
}

fn yield_bang(x: &mut Context, c: char) -> Option<Token> { Some(Bang) }

impl<I> Lexer2<I>
where I: Iterator<Item=char>
{
    fn new(mut iter: I) -> Self {
        let ch      = iter.next();
        let state   = match ch { Some(_) => Initial, None => AtEof };
        let context = Context {
            start:   Pos { byte: 0, line: 1, column: 1 },
            current: Pos { byte: 0, line: 1, column: 1 },
            buffer:  String::with_capacity(128),
            // interner
            // errors
        };
        Lexer2 { iter:iter, ch:ch, state:state, context:context }
    }

    fn lex(&mut self) -> Token {
        let     iter  = &mut self.iter;
        let mut ch    =      self.ch;
        let mut state =      self.state;

        loop {
            let (c, (next_state, consume, action))
                = lookup(&STATES[state as usize], ch);

            state = next_state;

            if consume {
                self.context.current.byte   += c.len_utf8();
                self.context.current.column += 1;
                ch = iter.next();
            }

            if let Some(func) = action {
                if let Some(token) = func(&mut self.context, c) {
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
fn lookup(entry: &StateEntry, ch: Option<char>) -> (char, (State, bool, Action))
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

// End new idea 2015-10-13

#[derive(Clone)]
pub struct Lexer<I>
where I: Iterator<Item=char>
{
    token: Token,               // current token
    input: IterLookahead<I>,    // remaining chars w/1 lookahead
    buf:   String,              // shared string builder
    start: Pos,                 // position of first char of    token
    end:   Pos,                 // position of first char after token
                                // ...also, position of self.ch()
    syms:  Box<SymbolTable>,
    errs:  u32,
    // symbols \__ a compilation context?
    // errors  /
}

impl<I> Lexer<I>
where I: Iterator<Item=char>
{
    fn new(iter: I) -> Self {
        Lexer {
            token: Eof,
            input: IterLookahead::new(iter),
            buf:   String::with_capacity(128),
            start: Pos { byte: 0, line: 1, column: 1 },
            end:   Pos { byte: 0, line: 1, column: 1 },
            syms:  Box::new(SymbolTable::new()),
            errs:  0
        }
    }

    fn lex(&mut self) -> Token {
        loop {
            let result = match self.ch() {
                None    => self.lex_eof(),
                Some(c) => match c {
                    ' ' | '\t'             => self.lex_space(),
                    '\r'                   => self.lex_cr(),
                    '\n'                   => self.lex_lf(),
                    '0'                    => self.lex_num_zero(),
                    '1'...'9'              => self.lex_num_nonzero(),
                    '_'                    => self.lex_id(),
                    _ if c.is_alphabetic() => self.lex_id(),
                    _                      => self.lex_other()
                }
            };
            if let Some(token) = result {
                self.token = token;
                return token;
            }
        }
    }

    fn lex_eof(&mut self) -> Option<Token> {
        self.start();
        Some(Eof)
    }

    fn lex_space(&mut self) -> Option<Token> {
        while let Some(c@' ') = self.advance().ch() {
            if c != ' ' && c != '\t' { break }
        }
        None
    }

    fn lex_cr(&mut self) -> Option<Token> {
        if let Some(c@'\n') = self.start().newline().ch() {
            self.consume();
        }
        Some(Eos)
    }

    fn lex_lf(&mut self) -> Option<Token> {
        self.start().newline();
        Some(Eos)
    }

    // Scan identifier (at alpha or _)
    //
    fn lex_id(&mut self) -> Option<Token> {
        let input = self.start().ch();
        self.buf.push(input.unwrap());

        while let Some(c) = self.advance().ch() {
            if c != '_' && !c.is_alphanumeric() { break; }
            self.buf.push(c);
        }

        let sym = self.syms.intern(&self.buf);
        self.buf.clear();
        Some(Id(sym))
    }

    // Scan number (at initial 0)
    //
    fn lex_num_zero(&mut self) -> Option<Token> {
        match self.start().advance().ch() {
            Some(c) => match c {
                '0'...'9' => { self          .lex_num_zero_digit (c)     },
                '_'       => { self.advance().lex_num_rest       (10, 0) },
                'x'       => { self.advance().lex_num_radix      (16   ) },
                'o'       => { self.advance().lex_num_radix      ( 8   ) },
                'b'       => { self.advance().lex_num_radix      ( 2   ) },
                 _        => { self          .lex_after_num      ( c, 0) }
            },
            None => Some(Int(0))
        }
    }

    // Scan number (at initial 1-9)
    //
    #[inline]
    fn lex_num_nonzero(&mut self) -> Option<Token> {
        let val = self.ch().unwrap().to_digit(10).unwrap();
        self.start().advance().lex_num_rest(10, val)
    }

    // Scan number (after initial 0, at digit)
    //
    #[inline]
    fn lex_num_zero_digit(&mut self, c: char) -> Option<Token> {
        self.lex_num_radix_digit(10, c.to_digit(10).unwrap())
    }

    // Scan number (at first char after any radix prefix)
    //
    fn lex_num_radix(&mut self, radix: u32) -> Option<Token> {
        let mut input = self.ch();

        // Ignore leading _
        while let Some('_') = input {
            input = self.advance().ch();
        }

        // Require a digit
        if let Some(c) = input {
            if let Some(d) = c.to_digit(radix) {
                return self.advance().lex_num_rest(radix, d)
            }
        }

        // Eof doesn't make sense here
        self.err_int_format()
    }

    // Scan number (at first digit of known radix)
    //
    fn lex_num_radix_digit(&mut self, radix: u32, mut value: u32) -> Option<Token> {
        loop {
            let input = self.advance().ch();
            match input {
                Some('_') => {},
                Some( c ) => match c.to_digit(radix) {
                    Some(d) => value = value * radix + d,
                    _       => return self.lex_after_num(c, value)
                },
                _ => return Some(Int(value))
            }
        }
    }

    // Scan number (after first digit of known radix)
    //
    fn lex_num_rest(&mut self, radix: u32, mut value: u32) -> Option<Token> {
        let mut input = self.ch();

        // Match digits or _
        loop {
        //    while let Some('_') = input {
        //        input = self.advance().ch();
        //    }

        //    if let Some(c) = input {
        //        if let Some(d) = c.to_digit(radix) {
        //            input = self.advance().ch();
        //            value = value * radix + d;
        //            continue;
        //        } else {
        //            return self.lex_after_num(c, value);
        //        }
        //    }

        //    // Eof is OK here
        //    return Some(Int(value))
        //}

            match input {
                Some('_') => {
                    input = self.advance().ch();
                },
                Some(c) => {
                    match c.to_digit(radix) {
                        Some(d) => {
                            input = self.advance().ch();
                            value = value * radix + d;
                        },
                        _ => return self.lex_after_num(c, value)
                    }
                },
                None => return Some(Int(value))
            }
        }
    }

    // Finish scanning number (at char after num)
    //
    fn lex_after_num(&mut self, c: char, val: u32) -> Option<Token> {
        // OK if at valid char
        if !c.is_digit(10) && !c.is_alphabetic() {
            return Some(Int(val));
        }

        // Eat invalid chars
        while let Some(c) = self.advance().ch() {
            if !c.is_digit(10) && !c.is_alphabetic() {
                break;
            }
        }

        return self.err_int_format();
    }

    fn lex_other(&mut self) -> Option<Token> {
        self.advance().ch();
        self.errs += 1; // TODO: error message
        None
    }

    #[inline]
    fn ch(&self) -> Option<char> {
        self.input.current()
    }

    // Consumes the current character without advancing position.
    #[inline]
    fn consume(&mut self) -> &mut Self {
        self.end.byte += self.ch().unwrap().len_utf8();
        self.input.next();
        self
    }

    // Consumes the current character and advances position to the next column.
    #[inline]
    fn advance(&mut self) -> &mut Self {
        self.consume();
        self.end.column += 1;
        self
    }

    // Consumes the current character and advances position to the next line.
    #[inline]
    fn newline(&mut self) -> &mut Self {
        self.consume();
        self.end.line   += 1;
        self.end.column  = 1;
        self
    }

    // Sets the current position as the start of a token.
    #[inline]
    fn start(&mut self) -> &mut Self {
        self.start = self.end;
        self
    }

    // Adds an error message
    #[inline]
    fn error<S>(&mut self, msg: S) -> Option<Token> where S: Into<String> {
        println!("{}", msg.into());
        None
    }

    fn err_int_format(&mut self) -> Option<Token> {
        self.error("Invalid integer literal.")
    }
}

impl<I> Lookahead for Lexer<I>
where I: Iterator<Item=char>
{
    type Item = Token;

    #[inline]
    fn current(&self) -> Token {
        self.token
    }

    #[inline]
    fn next(&mut self) -> Token {
        self.lex();
        self.token
    }
}

#[derive(Clone)]
struct IterLookahead<I> where I: Iterator, I::Item: Copy {
    item: Option<I::Item>,
    iter: I
}

impl<I> IterLookahead<I> where I: Iterator, I::Item: Copy {
    #[inline]
    fn new(mut iter: I) -> Self {
        let item = iter.next();
        IterLookahead { item: item, iter: iter }
    }
}

impl<I> Lookahead for IterLookahead<I> where I: Iterator, I::Item: Copy {
    type Item = Option<I::Item>;

    #[inline]
    fn current(&self) -> Self::Item {
        self.item
    }

    #[inline]
    fn next(&mut self) -> Self::Item {
        let item = self.iter.next();
        self.item = item;
        item
    }
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
        lex(  "\n", Eos, (1, 1), (2, 1), 0);
        lex("\r\n", Eos, (1, 1), (2, 1), 0);
        lex("\r"  , Eos, (1, 1), (2, 1), 0);
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
        let mut lexer = Lexer::new(s.chars());
        let     token = lexer.lex();

        assert_eq!(token,       t);
        assert_eq!(lexer.token, t);
        assert_eq!((lexer.start.line, lexer.start.column), start);
        assert_eq!((lexer.end  .line, lexer.end  .column), end  );
        assert_eq!(lexer.errs, errs);
    }

    fn lex_id(s: &str, name: &str, start: (u32, u32), end: (u32, u32), errs: u32)
    {
        let mut lexer = Lexer::new(s.chars());

        match lexer.lex() {
            Id(sym) => assert_eq!(lexer.syms.get(sym).name, name),
            _       => panic!("Did not return an identifier.")
        }

        assert_eq!((lexer.start.line, lexer.start.column), start);
        assert_eq!((lexer.end  .line, lexer.end  .column), end  );
        assert_eq!(lexer.errs, errs);
    }
}

