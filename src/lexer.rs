use char::*;
use char::CharClass::*;
use symbol::Symbol;

#[derive(Clone, Copy, PartialEq, Eq, Debug )]
pub struct Pos {
    pub offset: usize,  // 0-based byte offset
    pub line:   i32,    // 1-based line number
    pub column: i32,    // 1-based column number
}

impl Pos {
    #[inline]
    fn fwd(&mut self, c: char) {
        self.offset += c.len_utf8();
    }

    #[inline]
    fn fwd_col(&mut self, c: char) {
        self.fwd(c);
        self.column += 1;
    }

    #[inline]
    fn fwd_line(&mut self, c: char) {
        self.fwd(c);
        self.line   += 1;
        self.column  = 1;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug )]
pub enum Token {
    Id      (Symbol),
    IntLit  (i32),
    End
}
use self::Token::*;

pub trait Cursor {
    type Item;
    fn current(&self) -> Self::Item;
    fn advance(&mut self);
}

#[derive(Clone)]
pub struct Lexer<I>
where I: Iterator<Item=char>
{
    token: Token,       // current token
    input: Scanner<I>,  // remaining chars w/1 lookahead
    buf:   String,      // shared string builder
    start: Pos,         // position of first char of    token
    end:   Pos,         // position of first char after token
}

impl<I> Lexer<I>
where I: Iterator<Item=char>
{
    fn new(iter: I) -> Self {
        Lexer {
            token: End,
            input: Scanner::new(iter),
            buf:   String::with_capacity(128),
            start: Pos { offset: 0, line: 1, column: 1 },
            end:   Pos { offset: 0, line: 1, column: 1 }
        }
    }

    #[inline]
    fn lex(&mut self) {
        loop {
            match self.input.current() {
                (Eof, _) => {
                    self.start = self.end;
                    self.token = End;
                    return;
                },
                (Space, c) => {
                    self.end.fwd_col(c);
                    self.input.advance();
                    self.start = self.end;
                    continue;
                },
                (CR, c) => {
                    self.end.fwd_line(c);
                    self.input.advance();
                    self.start = self.end;
                    if let (LF, c) = self.input.current() {
                        self.end.fwd(c);
                        self.input.advance();
                    }
                    continue;
                }
                (LF, c) => {
                    self.end.fwd_line(c);
                    self.input.advance();
                    self.start = self.end;
                },
                (Alpha, c) => {
                    self.lex_id();
                }
                _ => return
            }
        }
    }

    fn lex_id(&mut self) {
        // we know first char is OK already

        let buf = &mut self.buf;

        buf.push(self.input.current().1);
        self.input.advance();

        loop {
            match self.input.current() {
                (Alpha, c) | (Digit, c) => {
                    buf.push(c);
                    self.input.current();
                },
                _ => break
            }
        }

        self.token = IntLit(42);
    }

    #[inline]
    fn advance(&mut self) {
        let (_, c) = self.input.current();
        self.end.fwd_col(c);
        self.input.advance();
    }
}

impl<I> Cursor for Lexer<I>
where I: Iterator<Item=char>
{
    type Item = Token;

    #[inline]
    fn current(&self) -> Token { self.token }
    fn advance(&mut self) { self.lex(); }
}

#[derive(Clone)]
struct Scanner<I> where I: Iterator<Item=char> {
    item: (CharClass, char),
    iter: I
}

impl<I> Scanner<I> where I: Iterator<Item=char> {
    #[inline]
    fn new(iter: I) -> Self {
        Scanner {
            item: SCANNER_EOF,
            iter: iter
        }
    }
}

impl<I> Cursor for Scanner<I> where I: Iterator<Item=char> {
    type Item = (CharClass, char);

    #[inline]
    fn current(&self) -> Self::Item {
        self.item
    }

    #[inline]
    fn advance(&mut self) {
        self.item = match self.iter.next() {
            Some(c) => c.classify(),
            None    => SCANNER_EOF
        }
    }
}

const SCANNER_EOF: (CharClass, char) = (Eof, '\n');

