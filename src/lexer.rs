use symbol::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug )]
pub struct Pos {
    pub byte:   usize,  // 0-based byte offset
    pub line:   u32,    // 1-based line number
    pub column: u32,    // 1-based column number
}

#[derive(Clone, Copy, PartialEq, Eq, Debug )]
pub enum Token {
    Id      (Symbol),
    IntLit  (i32),
    Eos,                // End of statement
    Eof,                // End of file
}
use self::Token::*;

pub trait Lookahead {
    type Item;
    fn current (&    self) -> Self::Item;
    fn next    (&mut self) -> Self::Item;
}

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
    // symbols \__ a compilation context?
    // errors  /
}

impl<I> Lexer<I>
where I: Iterator<Item=char>
{
    #[inline]
    fn new(iter: I) -> Self {
        Lexer {
            token: Eof,
            input: IterLookahead::new(iter),
            buf:   String::with_capacity(128),
            start: Pos { byte: 0, line: 1, column: 1 },
            end:   Pos { byte: 0, line: 1, column: 1 },
            syms:  Box::new(SymbolTable::new())
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
        while let Some(c@' ') = self.right().next_ch() {
            if c != ' ' && c != '\t' { break }
        }
        None
    }

    fn lex_cr(&mut self) -> Option<Token> {
        if let Some(c@'\n') = self.start().down().next_ch() {
            self.next_ch();
        }
        Some(Eos)
    }

    fn lex_lf(&mut self) -> Option<Token> {
        self.start().down().next_ch();
        Some(Eos)
    }

    fn lex_id(&mut self) -> Option<Token> {
        // we know first char is OK already
        let c = self.start().ch().unwrap();
        self.buf.push(c);

        while let Some(c) = self.right().next_ch() {
            if !c.is_alphanumeric() { break }
            self.buf.push(c);
        }

        let sym = self.syms.intern(&self.buf);
        Some(Id(sym))
    }

    fn lex_other(&mut self) -> Option<Token> {
        self.right().next_ch();
        // TODO: error message
        None
    }

    #[inline]
    fn ch(&self) -> Option<char> {
        self.input.current()
    }

    #[inline]
    fn next_ch(&mut self) -> Option<char> {
        match self.ch() {
            Some(c) => {
                self.end.byte += c.len_utf8();
                self.input.next()
            },
            None => None
        }
    }

    // Sets the current line/col position as the start of a token
    #[inline]
    fn start(&mut self) -> &mut Self {
        self.start = self.end;
        self
    }

    // Advances line/col position to the next column
    #[inline]
    fn right(&mut self) -> &mut Self {
        self.end.column += 1;
        self
    }

    // Advances line/col position to column 1 of the next line
    #[inline]
    fn down(&mut self) -> &mut Self {
        self.end.line   += 1;
        self.end.column  = 1;
        self
    }

    // Sets the current token
    #[inline]
    fn produce(&mut self, t: Token) -> &mut Self {
        self.token = t;
        self
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
    fn new(iter: I) -> Self {
        IterLookahead { item: None, iter: iter }
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

