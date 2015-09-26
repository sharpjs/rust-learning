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
    Int     (u32),
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
                    '0'                    => self.start().advance().lex_num(),
                    '1'...'9'              => self.start().advance().lex_num_radix(10),
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

    fn lex_id(&mut self) -> Option<Token> {
        // we know first char is OK already
        let c = self.start().ch().unwrap();
        self.buf.push(c);

        while let Some(c) = self.advance().ch() {
            if !c.is_alphanumeric() { break }
            self.buf.push(c);
        }

        let sym = self.syms.intern(&self.buf);
        Some(Id(sym))
    }

    // Parse number (current char is a zero)
    fn lex_num(&mut self) -> Option<Token> {
        match self.start().advance().ch() {
            Some(c) => match c {
                '0'...'9' => {                 self.lex_num_radix(10) },
                'x'       => { self.advance(); self.lex_num_radix(16) },
                'o'       => { self.advance(); self.lex_num_radix( 8) },
                'b'       => { self.advance(); self.lex_num_radix( 2) },
                _         => self.error("Invalid integer format")
            },
            _ => self.error("")
        }
    }

    // Parse number after a radix prefix
    fn lex_num_radix(&mut self, radix: u32) -> Option<Token> {
        let mut input = self.ch();
        let mut value: u32;

        // Allow leading _ but require at least one digit
        loop {
            match input {
                Some('_') => {
                    input = self.advance().ch();
                    continue;
                },
                Some(c) => {
                    if let Some(d) = c.to_digit(radix) {
                        input = self.advance().ch();
                        value = d;
                        break;
                    }
                },
                _ => {}
            }
            return None; // Error: incomplete integer literal
        }

        // Match trailing digits or _
        loop {
            match input {
                Some('_') => {
                    input = self.advance().ch();
                    continue;
                },
                Some(c) => {
                    if let Some(d) = c.to_digit(radix) {
                        input = self.advance().ch();
                        value = value * radix + d;
                        continue;
                    }
                    if c.is_alphabetic() || c.is_digit(10) {
                        // eat invalid chars
                        return None; // Error: Invalid integer literal
                    }
                },
                _ => {}
            }
            return Some(Int(value));
        }
    }

    fn lex_other(&mut self) -> Option<Token> {
        self.advance().ch();
        // TODO: error message
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
    fn error<S>(&mut self, msg: S) -> Option<Token>
    where S: Into<String> {
        None
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

