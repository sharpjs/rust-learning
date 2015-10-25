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
use std::rc::Rc;

use interner::*;
use message::*;
use message::Message::*;

// -----------------------------------------------------------------------------
// Tokens

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
    BraceL,         // {
    BraceR,         // }
    ParenL,         // (
    ParenR,         // )
    BracketL,       // [
    BracketR,       // ]
    Dot,            // .
    At,             // @
    PlusPlus,       // ++
    MinusMinus,     // --
    Bang,           // !
    Tilde,          // ~
    Question,       // ?
    Star,           // *
    Slash,          // /
    Percent,        // %
    Plus,           // +
    Minus,          // -
    LessLess,       // <<
    MoreMore,       // >>
    Ampersand,      // &
    Caret,          // ^
    Pipe,           // |
    DotTilde,       // .~
    DotBang,        // .!
    DotEqual,       // .=
    DotQuestion,    // .?
    LessMore,       // <>
    EqualEqual,     // ==
    BangEqual,      // !=
    Less,           // <
    More,           // >
    LessEqual,      // <=
    MoreEqual,      // >=
    EqualArrow,     // =>
    MinusArrow,     // ->
    Equal,          // =
    Colon,          // :
    Comma,          // ,
    Eos,            // End of statement
    Eof,            // End of file
    Error (Message) // Lexical error
}
use self::Token::*;

// -----------------------------------------------------------------------------
// Misc Types

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
    AfterDot, AfterPlus, AfterMinus,
    AfterLess, AfterMore,  AfterEqual, AfterBang,
    AtEof
}
use self::State::*;

type TransitionSet = (
    [u8; 128],      // Map from 7-bit char to transition index
    &'static [(     // Transition array
        State,      // - next state
        bool,       // - true => consume char
        Action      // - action code
    )]
);

// -----------------------------------------------------------------------------
// Action Codes

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Action {
    Skip,
    Space,
    Newline,
    YieldEos,
    YieldEof,

    AccumNumDec,
    AccumNumHexDig,
    AccumNumHexUc,
    AccumNumHexLc,
    AccumNumOct,
    AccumNumBin,
    YieldNum,

    StartEsc,
    AccumStr,
    AccumStrEscNul,
    AccumStrEscLf,
    AccumStrEscCr,
    AccumStrEscTab,
    AccumStrEscChar,
    AccumStrEscNum,
    AccumStrEscHexDig,
    AccumStrEscHexUc,
    AccumStrEscHexLc,
    YieldChar,
    YieldStr,
    YieldId,

    YieldBraceL,
    YieldBraceR,
    YieldParenL,
    YieldParenR,
    YieldBracketL,
    YieldBracketR,
    YieldDot,
    YieldAt,
    YieldPlusPlus,
    YieldMinusMinus,
    YieldBang,
    YieldTilde,
    YieldQuestion,
    YieldStar,
    YieldSlash,
    YieldPercent,
    YieldPlus,
    YieldMinus,
    YieldLessLess,
    YieldMoreMore,
    YieldAmpersand,
    YieldCaret,
    YieldPipe,
    YieldDotTilde,
    YieldDotBang,
    YieldDotEqual,
    YieldDotQuestion,
    YieldLessMore,
    YieldEqualEqual,
    YieldBangEqual,
    YieldLess,
    YieldMore,
    YieldLessEqual,
    YieldMoreEqual,
    YieldEqualArrow,
    YieldMinusArrow,
    YieldEqual,
    YieldColon,
    YieldComma,

    ErrorInvalid,
    ErrorNumInvalid,
    ErrorNumOverflow,
    ErrorCharUnterm,
    ErrorCharLength,
    ErrorEscOverflow,
    ErrorStrUnterm,
    ErrorEscUnterm,
    ErrorEscInvalid,
}
use self::Action::*;

// -----------------------------------------------------------------------------
// Lexer

pub struct Lexer<I>
where I: Iterator<Item=char>
{
    iter:       I,                      // remaining chars
    ch:         Option<char>,           // char  after previous token
    state:      State,                  // state after previous token
    context:    Context                 // context object give to actions
}

impl<I> Lexer<I>
where I: Iterator<Item=char>
{
    pub fn new(strings: Rc<Interner>, mut iter: I) -> Self {
        let ch = iter.next();

        Lexer {
            iter:    iter,
            ch:      ch,
            state:   match ch { Some(_) => Initial, None => AtEof },
            context: Context::new(strings)
        }
    }

    pub fn strings(&self) -> &Interner {
        &self.context.strings
    }

    pub fn lex(&mut self) -> (Pos, Token, Pos) {
        let mut ch    =      self.ch;
        let mut state =      self.state;
        let     iter  = &mut self.iter;
        let     l     = &mut self.context;

        println!("\nstate = {:?}", state);

        loop {
            // Lookup handler for this state and char
            let (c, (next_state, consume, action))
                = lookup(&STATES[state as usize], ch);

            println!("{:?} {:?} => {:?} {:?} {:?}", state, ch, c, next_state, consume);

            // Advance state machine
            state = next_state;

            // Consume character and get next
            if consume {
                l.current.byte   += c.len_utf8();
                l.current.column += 1;
                ch                  = iter.next();
            }

            // Action code helpers
            macro_rules! push {
                ( $s:expr ) => {{ self.state = state; state = $s }};
            }
            macro_rules! pop {
                () => {{ state = self.state }};
            }
            macro_rules! skip {
                ( $($e:expr);* ) => {{ $($e;)* continue }};
            }
            macro_rules! maybe {
                ( $($e:expr);+ ) => {
                    match { $($e);+ } { Some(e) => e, _ => continue }
                };
            }

            // Invoke action code
            let token = match action {
                // Space
                Skip                => skip! {                        },
                Space               => skip! {              l.start() },
                Newline             => skip! { l.newline(); l.start() },
                YieldEos            => Eos,
                YieldEof            => Eof,

                // Numbers
                AccumNumDec         => maybe! { l.num_add_dec     (c) },
                AccumNumHexDig      => maybe! { l.num_add_hex_dig (c) },
                AccumNumHexUc       => maybe! { l.num_add_hex_uc  (c) },
                AccumNumHexLc       => maybe! { l.num_add_hex_lc  (c) },
                AccumNumOct         => maybe! { l.num_add_oct     (c) },
                AccumNumBin         => maybe! { l.num_add_bin     (c) },
                YieldNum            => l.num_get(),

                // Strings & Chars
                AccumStr            => skip!  {         l.str_add             (  c ) },
                AccumStrEscNul      => skip!  { pop!(); l.str_add             ('\0') },
                AccumStrEscLf       => skip!  { pop!(); l.str_add             ('\n') },
                AccumStrEscCr       => skip!  { pop!(); l.str_add             ('\r') },
                AccumStrEscTab      => skip!  { pop!(); l.str_add             ('\t') },
                AccumStrEscChar     => skip!  { pop!(); l.str_add             (  c ) },
                AccumStrEscNum      => maybe! { pop!(); l.str_add_esc         (    ) },
                AccumStrEscHexDig   => maybe! { pop!(); l.str_add_esc_hex_dig (  c ) },
                AccumStrEscHexUc    => maybe! { pop!(); l.str_add_esc_hex_uc  (  c ) },
                AccumStrEscHexLc    => maybe! { pop!(); l.str_add_esc_hex_lc  (  c ) },
                StartEsc            => skip!  { push!(InEsc) },
                YieldStr            => l.str_get(),
                YieldChar           => l.str_get_char(),
                YieldId             => l.str_get_id_or_keyword(),

                // Simple Tokens
                YieldBraceL         => BraceL,
                YieldBraceR         => BraceR,
                YieldParenL         => ParenL,
                YieldParenR         => ParenR,
                YieldBracketL       => BracketL,
                YieldBracketR       => BracketR,
                YieldDot            => Dot,
                YieldAt             => At,
                YieldPlusPlus       => PlusPlus,
                YieldMinusMinus     => MinusMinus,
                YieldBang           => Bang,
                YieldTilde          => Tilde,
                YieldQuestion       => Question,
                YieldStar           => Star,
                YieldSlash          => Slash,
                YieldPercent        => Percent,
                YieldPlus           => Plus,
                YieldMinus          => Minus,
                YieldLessLess       => LessLess,
                YieldMoreMore       => MoreMore,
                YieldAmpersand      => Ampersand,
                YieldCaret          => Caret,
                YieldPipe           => Pipe,
                YieldDotTilde       => DotTilde,
                YieldDotBang        => DotBang,
                YieldDotEqual       => DotEqual,
                YieldDotQuestion    => DotQuestion,
                YieldLessMore       => LessMore,
                YieldEqualEqual     => EqualEqual,
                YieldBangEqual      => BangEqual,
                YieldLess           => Less,
                YieldMore           => More,
                YieldLessEqual      => LessEqual,
                YieldMoreEqual      => MoreEqual,
                YieldEqualArrow     => EqualArrow,
                YieldMinusArrow     => MinusArrow,
                YieldEqual          => Equal,
                YieldColon          => Colon,
                YieldComma          => Comma,

                // Errors
                ErrorInvalid        => Error(Lex_Invalid),
                ErrorNumInvalid     => Error(Lex_NumInvalid),
                ErrorNumOverflow    => Error(Lex_NumOverflow),
                ErrorCharUnterm     => Error(Lex_CharUnterminated),
                ErrorCharLength     => Error(Lex_CharLength),
                ErrorEscOverflow    => Error(Lex_EscOverflow),
                ErrorStrUnterm      => Error(Lex_StrUnterminated),
                ErrorEscUnterm      => Error(Lex_EscUnterminated),
                ErrorEscInvalid     => Error(Lex_EscInvalid),
            };

            // Remember state for next invocation
            self.ch    = ch;
            self.state = state;

            // Yield
            let start = l.start; l.start();
            let end   = l.current;
            return (start, token, end);
        }
    }
}

// -----------------------------------------------------------------------------
// Adapter for LALRPOP

impl<I> Iterator for Lexer<I>
where I: Iterator<Item=char>
{
    type Item = Result<(Pos, Token, Pos), (Pos, Message)>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lex() {
            (_,   Eof,        _) => None,
            (pos, Error(msg), _) => Some(Err( (pos, msg) )),
            triple               => Some(Ok (   triple   )),
        }
    }
}

// -----------------------------------------------------------------------------
// State Transition Table

#[inline]
fn lookup(entry: &TransitionSet, ch: Option<char>) -> (char, (State, bool, Action))
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

const STATES: &'static [TransitionSet] = &[
    // Initial
    ([
        x, x, x, x, x, x, x, x,  x, 2, 3, x, x, 2, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        2,19, 9, x, x,24,27, 8, 13,14,22,25,34,26,17,23, //  !"#$%&' ()*+,-./
        6, 7, 7, 7, 7, 7, 7, 7,  7, 7,33, 4,30,32,31,21, // 01234567 89:;<=>?
       18, 5, 5, 5, 5, 5, 5, 5,  5, 5, 5, 5, 5, 5, 5, 5, // @ABCDEFG HIJKLMNO
        5, 5, 5, 5, 5, 5, 5, 5,  5, 5, 5,15, x,16,28, 5, // PQRSTUVW XYZ[\]^_
       10, 5, 5, 5, 5, 5, 5, 5,  5, 5, 5, 5, 5, 5, 5, 5, // `abcdefg hijklmno
        5, 5, 5, 5, 5, 5, 5, 5,  5, 5, 5,11,29,12,20, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //              State       Next?  Action
        /*  0: eof */ ( AtEof,      false, YieldEof       ),
        /*  1: ??? */ ( AtEof,      false, ErrorInvalid   ),
        /*  2: \s  */ ( Initial,    true,  Space          ),
        /*  3: \n  */ ( AfterEos,   true,  Newline        ),
        /*  4:  ;  */ ( AfterEos,   true,  Skip           ),
        /*  5: id0 */ ( InId,       true,  AccumStr       ),
        /*  6:  0  */ ( AfterZero,  true,  Skip           ),
        /*  7: 1-9 */ ( InNumDec,   true,  AccumNumDec    ),
        /*  8:  '  */ ( InChar,     true,  Skip           ),
        /*  9:  "  */ ( InStr,      true,  Skip           ),
        /* 10:  `  */ ( InStr,      true,  Skip           ), // TODO InTildeStr
        /* 11:  {  */ ( Initial,    true,  YieldBraceL    ),
        /* 12:  }  */ ( Initial,    true,  YieldBraceR    ),
        /* 13:  (  */ ( Initial,    true,  YieldParenL    ),
        /* 14:  )  */ ( Initial,    true,  YieldParenR    ),
        /* 15:  [  */ ( Initial,    true,  YieldBracketL  ),
        /* 16:  ]  */ ( Initial,    true,  YieldBracketR  ),
        /* 17:  .  */ ( AfterDot,   true,  Skip           ),
        /* 18:  @  */ ( Initial,    true,  YieldAt        ),
        /* 19:  !  */ ( AfterBang,  true,  Skip           ),
        /* 20:  ~  */ ( Initial,    true,  YieldTilde     ),
        /* 21:  ?  */ ( Initial,    true,  YieldQuestion  ),
        /* 22:  *  */ ( Initial,    true,  YieldStar      ),
        /* 23:  /  */ ( Initial,    true,  YieldSlash     ),
        /* 24:  %  */ ( Initial,    true,  YieldPercent   ),
        /* 25:  +  */ ( AfterPlus,  true,  Skip           ),
        /* 26:  -  */ ( AfterMinus, true,  Skip           ),
        /* 27:  &  */ ( Initial,    true,  YieldAmpersand ),
        /* 28:  ^  */ ( Initial,    true,  YieldCaret     ),
        /* 29:  |  */ ( Initial,    true,  YieldPipe      ),
        /* 30:  <  */ ( AfterLess,  true,  Skip           ),
        /* 31:  >  */ ( AfterMore,  true,  Skip           ),
        /* 32:  =  */ ( AfterEqual, true,  Skip           ),
        /* 33:  :  */ ( Initial,    true,  YieldColon     ),
        /* 34:  ,  */ ( Initial,    true,  YieldComma     ),
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
        //             State     Next?  Action
        /* 0: eof */ ( AtEof,    false, YieldEos ),
        /* 1: ??? */ ( Initial,  false, YieldEos ),
        /* 2: \s; */ ( AfterEos, true,  Space    ),
        /* 3: \n  */ ( AfterEos, true,  Newline  ),
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
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,   false, YieldId  ),
        /* 1: ??? */ ( Initial, false, YieldId  ),
        /* 2: id  */ ( InId,    true,  AccumStr ),
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
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,    false, YieldNum        ),
        /* 1: ??? */ ( AtEof,    false, ErrorNumInvalid ),
        /* 2: 0-9 */ ( InNumDec, true,  AccumNumDec     ),
        /* 3:  _  */ ( InNumDec, true,  Skip            ),
        /* 4:  x  */ ( InNumHex, true,  Skip            ),
        /* 5:  o  */ ( InNumOct, true,  Skip            ),
        /* 6:  b  */ ( InNumBin, true,  Skip            ),
        /* 7: \s  */ ( Initial,  true,  YieldNum        ),
        /* 8: opr */ ( Initial,  false, YieldNum        ),
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
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,    false, YieldNum        ),
        /* 1: ??? */ ( AtEof,    false, ErrorNumInvalid ),
        /* 2: 0-9 */ ( InNumDec, true,  AccumNumDec     ),
        /* 3:  _  */ ( InNumDec, true,  Skip            ),
        /* 4: \s  */ ( Initial,  true,  YieldNum        ),
        /* 5: opr */ ( Initial,  false, YieldNum        ),
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
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,    false, YieldNum        ),
        /* 1: ??? */ ( AtEof,    false, ErrorNumInvalid ),
        /* 2: 0-9 */ ( InNumHex, true,  AccumNumHexDig  ),
        /* 3: A-F */ ( InNumHex, true,  AccumNumHexUc   ),
        /* 4: a-f */ ( InNumHex, true,  AccumNumHexLc   ),
        /* 5:  _  */ ( InNumHex, true,  Skip            ),
        /* 6: \s  */ ( Initial,  true,  YieldNum        ),
        /* 7: opr */ ( Initial,  false, YieldNum        ),
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
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,    false, YieldNum        ),
        /* 1: ??? */ ( AtEof,    false, ErrorNumInvalid ),
        /* 2: 0-7 */ ( InNumOct, true,  AccumNumOct     ),
        /* 3:  _  */ ( InNumOct, true,  Skip            ),
        /* 6: \s  */ ( Initial,  true,  YieldNum        ),
        /* 4: opr */ ( Initial,  false, YieldNum        ),
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
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,    false, YieldNum        ),
        /* 1: ??? */ ( AtEof,    false, ErrorNumInvalid ),
        /* 2: 0-1 */ ( InNumBin, true,  AccumNumBin     ),
        /* 3:  _  */ ( InNumBin, true,  Skip            ),
        /* 6: \s  */ ( Initial,  true,  YieldNum        ),
        /* 4: opr */ ( Initial,  false, YieldNum        ),
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
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,     false, ErrorCharUnterm ),
        /* 1: ??? */ ( AtCharEnd, true,  AccumStr        ),
        /* 2:  \  */ ( AtCharEnd, true,  StartEsc        ),
        /* 3:  '  */ ( AtEof,     false, ErrorCharLength ),
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
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,   false, ErrorCharUnterm ),
        /* 1: ??? */ ( AtEof,   false, ErrorCharLength ),
        /* 2:  '  */ ( Initial, true,  YieldChar       ),
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
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,   false, ErrorStrUnterm ),
        /* 1: ??? */ ( InStr,   true,  AccumStr       ),
        /* 2:  \  */ ( InStr,   true,  StartEsc       ),
        /* 3:  "  */ ( Initial, true,  YieldStr       ),
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
        //             State       Next?  Action
        /*  0: eof */ ( AtEof,     false, ErrorEscUnterm  ),
        /*  1: ??? */ ( AtEof,     false, ErrorEscInvalid ),
        /*  2:  0  */ ( InStr,     true,  AccumStrEscNul  ), // pops state
        /*  3:  n  */ ( InStr,     true,  AccumStrEscLf   ), // pops state
        /*  4:  r  */ ( InStr,     true,  AccumStrEscCr   ), // pops state
        /*  5:  t  */ ( InStr,     true,  AccumStrEscTab  ), // pops state
        /*  6:  \  */ ( InStr,     true,  AccumStrEscChar ), // pops state
        /*  7:  '  */ ( InStr,     true,  AccumStrEscChar ), // pops state
        /*  8:  "  */ ( InStr,     true,  AccumStrEscChar ), // pops state
        /*  9:  x  */ ( AtEscHex0, true,  Skip            ),
        /* 10:  u  */ ( AtEscUni0, true,  Skip            ),
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
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,     false, ErrorEscUnterm  ),
        /* 1: ??? */ ( AtEof,     false, ErrorEscInvalid ),
        /* 2: 0-9 */ ( AtEscHex1, true,  AccumNumHexDig  ),
        /* 3: A-F */ ( AtEscHex1, true,  AccumNumHexUc   ),
        /* 4: a-f */ ( AtEscHex1, true,  AccumNumHexLc   ),
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
        //             State    Next?  Action
        /* 0: eof */ ( AtEof, false, ErrorEscUnterm    ),
        /* 1: ??? */ ( AtEof, false, ErrorEscInvalid   ),
        /* 2: 0-9 */ ( InStr, true,  AccumStrEscHexDig ), // pops state
        /* 3: A-F */ ( InStr, true,  AccumStrEscHexUc  ), // pops state
        /* 4: a-f */ ( InStr, true,  AccumStrEscHexLc  ), // pops state
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
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,     false, ErrorEscUnterm  ),
        /* 1: ??? */ ( AtEof,     false, ErrorEscInvalid ),
        /* 2:  {  */ ( AtEscUni1, true,  Skip            ),
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
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,     false, ErrorEscUnterm  ),
        /* 1: ??? */ ( AtEof,     false, ErrorEscInvalid ),
        /* 2: 0-9 */ ( AtEscUni1, true,  AccumNumHexDig  ),
        /* 3: A-F */ ( AtEscUni1, true,  AccumNumHexUc   ),
        /* 4: a-f */ ( AtEscUni1, true,  AccumNumHexLc   ),
        /* 5:  }  */ ( InStr,     true,  AccumStrEscNum  ), // pops state
    ]),

    // AfterDot <( . )> : after a dot
    ([
        x, x, x, x, x, x, x, x,  x, 2, x, x, x, 2, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        2, 4, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        x, x, x, x, x, x, x, x,  x, x, x, x, x, 5, x, 6, // 01234567 89:;<=>?
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, 3, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,   false, YieldDot         ),
        /* 1: ??? */ ( Initial, false, YieldDot         ),
        /* 2: \s  */ ( Initial, true,  YieldDot         ),
        /* 3:  ~  */ ( Initial, true,  YieldDotTilde    ),
        /* 4:  !  */ ( Initial, true,  YieldDotBang     ),
        /* 5:  =  */ ( Initial, true,  YieldDotEqual    ),
        /* 6:  ?  */ ( Initial, true,  YieldDotQuestion ),
    ]),

    // AfterPlus <( + )> : after a plus sign
    ([
        x, x, x, x, x, x, x, x,  x, 2, x, x, x, 2, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        2, x, x, x, x, x, x, x,  x, x, x, 3, x, x, x, x, //  !"#$%&' ()*+,-./
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // 01234567 89:;<=>?
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,   false, YieldPlus     ),
        /* 1: ??? */ ( Initial, false, YieldPlus     ),
        /* 2: \s  */ ( Initial, true,  YieldPlus     ),
        /* 3:  +  */ ( Initial, true,  YieldPlusPlus ),
    ]),

    // AfterMinus <( - )> : after a minus sign
    ([
        x, x, x, x, x, x, x, x,  x, 2, x, x, x, 2, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        2, x, x, x, x, x, x, x,  x, x, x, x, x, 3, x, x, //  !"#$%&' ()*+,-./
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, 4, x, // 01234567 89:;<=>?
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,   false, YieldMinus      ),
        /* 1: ??? */ ( Initial, false, YieldMinus      ),
        /* 2: \s  */ ( Initial, true,  YieldMinus      ),
        /* 3:  -  */ ( Initial, true,  YieldMinusMinus ),
        /* 4:  >  */ ( Initial, true,  YieldMinusArrow ),
    ]),

    // AfterLess <( < )> : after a less-than sign
    ([
        x, x, x, x, x, x, x, x,  x, 2, x, x, x, 2, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        2, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        x, x, x, x, x, x, x, x,  x, x, x, x, 3, 5, 4, x, // 01234567 89:;<=>?
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,   false, YieldLess      ),
        /* 1: ??? */ ( Initial, false, YieldLess      ),
        /* 2: \s  */ ( Initial, true,  YieldLess      ),
        /* 3:  <  */ ( Initial, true,  YieldLessLess  ),
        /* 4:  >  */ ( Initial, true,  YieldLessMore  ),
        /* 5:  =  */ ( Initial, true,  YieldLessEqual ),
    ]),

    // AfterMore <( > )> : after a greater-than sign
    ([
        x, x, x, x, x, x, x, x,  x, 2, x, x, x, 2, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        2, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        x, x, x, x, x, x, x, x,  x, x, x, x, 3, 5, 4, x, // 01234567 89:;<=>?
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,   false, YieldMore      ),
        /* 1: ??? */ ( Initial, false, YieldMore      ),
        /* 2: \s  */ ( Initial, true,  YieldMore      ),
        /* 3:  <  */ ( Initial, true,  YieldMoreMore  ), // TODO: exchange
        /* 4:  >  */ ( Initial, true,  YieldMoreMore  ),
        /* 5:  =  */ ( Initial, true,  YieldMoreEqual ),
    ]),

    // AfterEqual <( = )> : after an equal sign
    ([
        x, x, x, x, x, x, x, x,  x, 2, x, x, x, 2, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        2, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        x, x, x, x, x, x, x, x,  x, x, x, x, x, 4, 3, x, // 01234567 89:;<=>?
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,   false, YieldEqual      ),
        /* 1: ??? */ ( Initial, false, YieldEqual      ),
        /* 2: \s  */ ( Initial, true,  YieldEqual      ),
        /* 3:  >  */ ( Initial, true,  YieldEqualArrow ),
        /* 4:  =  */ ( Initial, true,  YieldEqualEqual ),
    ]),

    // AfterBang <( ! )> : after a bang mark
    ([
        x, x, x, x, x, x, x, x,  x, 2, x, x, x, 2, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        2, 4, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        x, x, x, x, x, x, x, x,  x, x, x, x, x, 3, x, x, // 01234567 89:;<=>?
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             State    Next?  Action
        /* 0: eof */ ( AtEof,     false, YieldBang      ),
        /* 1: ??? */ ( Initial,   false, YieldBang      ),
        /* 2: \s  */ ( Initial,   true,  YieldBang      ),
        /* 3:  =  */ ( Initial,   true,  YieldBangEqual ),
        /* 4:  !  */ ( AfterBang, false, YieldBang      ),
    ]),

//  // State <( prior chars )> : state description
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
        //             State  Next?  Action
//      /* 0: eof */ ( AtEof, false, Skip ),
//      /* 1: ??? */ ( AtEof, false, Skip ),
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
        //             State  Next?  Action
        /* 0: eof */ ( AtEof, false, YieldEof ),
    ]),
];

// -----------------------------------------------------------------------------
// Keywords

const KEYWORDS: &'static [(&'static str, Token)] = &[
    ( "type"     , KwType     ),
    ( "struct"   , KwStruct   ),
    ( "union"    , KwUnion    ),
    ( "if"       , KwIf       ),
    ( "else"     , KwElse     ),
    ( "loop"     , KwLoop     ),
    ( "while"    , KwWhile    ),
    ( "break"    , KwBreak    ),
    ( "continue" , KwContinue ),
    ( "return"   , KwReturn   ),
    ( "jump"     , KwJump     ),
];

// -----------------------------------------------------------------------------
// Context

struct Context {
    start:      Pos,                    // position of token start
    current:    Pos,                    // position of current character
    number:     u64,                    // number builder
    buffer:     String,                 // string builder
    strings:    Rc<Interner>,           // string interner
    keywords:   HashMap<StrId, Token>   // keyword table
}

impl Context {
    fn new(strings: Rc<Interner>) -> Self {
        let mut keywords = HashMap::new();
        for &(k, t) in KEYWORDS {
            keywords.insert(strings.intern(k), t);
        }

        Context {
            start:    Pos { byte: 0, line: 1, column: 1 },
            current:  Pos { byte: 0, line: 1, column: 1 },
            buffer:   String::with_capacity(128),
            number:   0,
            strings:  strings,
            keywords: keywords
        }
    }

    #[inline]
    fn start(&mut self) {
        self.start = self.current;
    }

    #[inline]
    fn newline(&mut self) {
        self.current.column = 1;
        self.current.line  += 1;
    }

    // Number actions

    #[inline]
    fn num_add_dec(&mut self, c: char) -> Option<Token> {
        self.num_add(10, int_from_dig(c))
    }

    #[inline]
    fn num_add_hex_dig(&mut self, c: char) -> Option<Token> {
        self.num_add(16, int_from_dig(c))
    }

    #[inline]
    fn num_add_hex_uc(&mut self, c: char) -> Option<Token> {
        self.num_add(16, int_from_hex_uc(c))
    }

    #[inline]
    fn num_add_hex_lc(&mut self, c: char) -> Option<Token> {
        self.num_add(16, int_from_hex_lc(c))
    }

    #[inline]
    fn num_add_oct(&mut self, c: char) -> Option<Token> {
        self.num_add(8, int_from_dig(c))
    }

    #[inline]
    fn num_add_bin(&mut self, c: char) -> Option<Token> {
        self.num_add(2, int_from_dig(c))
    }

    #[inline]
    fn num_add(&mut self, base: u8, digit: u8) -> Option<Token> {
        let mut n = self.number;

        n = match n.checked_mul(base as u64) {
            Some(n) => n,
            None    => return Some(Error(Lex_NumOverflow))
        };

        n = match n.checked_add(digit as u64) {
            Some(n) => n,
            None    => return Some(Error(Lex_NumOverflow))
        };

        self.number = n;
        None
    }

    #[inline]
    fn num_get(&mut self) -> Token {
        let n = self.number;
        self.number = 0;
        Int(n)
    }

    // Character/String Actions

    #[inline]
    fn str_add(&mut self, c: char) {
        self.buffer.push(c);
    }

    #[inline]
    fn str_add_esc(&mut self) -> Option<Token> {
        let n = self.number as u32;
        if  n > UNICODE_MAX { return Some(Error(Lex_EscOverflow)) }
        let c = unsafe { mem::transmute(n) };
        self.buffer.push(c);
        None
    }

    #[inline]
    fn str_add_esc_hex_dig(&mut self, c: char) -> Option<Token> {
        self.num_add_hex_dig(c)
            .or_else(|| self.str_add_esc())
    }

    #[inline]
    fn str_add_esc_hex_uc(&mut self, c: char) -> Option<Token> {
        self.num_add_hex_uc(c)
            .or_else(|| self.str_add_esc())
    }

    #[inline]
    fn str_add_esc_hex_lc(&mut self, c: char) -> Option<Token> {
        self.num_add_hex_lc(c)
            .or_else(|| self.str_add_esc())
    }

    #[inline]
    fn str_get(&mut self) -> Token {
        let s = self.strings.add(&self.buffer);
        self.buffer.clear();
        Str(s)
    }

    #[inline]
    fn str_get_char(&mut self) -> Token {
        let c = self.buffer.chars().next().unwrap();
        self.buffer.clear();
        Char(c)
    }

    #[inline]
    fn str_get_id_or_keyword(&mut self) -> Token {
        let id = self.strings.intern(&self.buffer);

        match self.keywords.get(&id) {
            Some(&k) => k,
            None     => Id(id)
        }
    }
}

const UNICODE_MAX: u32 = 0x10FFFF;

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

// -----------------------------------------------------------------------------
// Tests

#[cfg(test)]
mod tests {
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

    #[test]
    fn punctuation() {
        lex("{ } ( ) [ ] . @ ++ -- ! ~ ? * / % + - << >> & ^ | .~ .! .= .? \
             <> == != < > <= >= => -> = : ,")
            .yields(BraceL)     .yields(BraceR)     .yields(ParenL)    .yields(ParenR)
            .yields(BracketL)   .yields(BracketR)   .yields(Dot)       .yields(At)
            .yields(PlusPlus)   .yields(MinusMinus)
            .yields(Bang)       .yields(Tilde)      .yields(Question)
            .yields(Star)       .yields(Slash)      .yields(Percent)
            .yields(Plus)       .yields(Minus)
            .yields(LessLess)   .yields(MoreMore)
            .yields(Ampersand)  .yields(Caret)      .yields(Pipe)
            .yields(DotTilde)   .yields(DotBang)    .yields(DotEqual)  .yields(DotQuestion)
            .yields(LessMore)   .yields(EqualEqual) .yields(BangEqual)
            .yields(Less)       .yields(More)       .yields(LessEqual) .yields(MoreEqual)
            .yields(EqualArrow) .yields(MinusArrow) .yields(Equal)
            .yields(Colon)      .yields(Comma)
            .yields(Eof);
    }

    // Test Harness

    use std::rc::Rc;
    use std::str::Chars;
    use ::interner::*;

    struct LexerHarness<'a> (Lexer<Chars<'a>>);

    fn lex(input: &str) -> LexerHarness {
        let chars   = input.chars();
        let strings = Rc::new(Interner::new());
        let lexer   = Lexer::new(strings, chars);
        LexerHarness(lexer)
    }

    impl<'a> LexerHarness<'a> {
        fn yields(&mut self, token: Token) -> &mut Self {
            assert_eq!(token, self.0.lex().1);
            self
        }

        fn yields_id(&mut self, name: &str) -> &mut Self {
            let token = self.0.lex().1;
            match token {
                Id(n) => assert_eq!(name, *self.0.context.strings.get(n)),
                _     => panic!("lex() did not yield an identifier.")
            };
            self
        }

        fn yields_error(&mut self) -> &mut Self {
            let token = self.0.lex().1;
            match token {
                Error(_) => {}, // expected
                _        => panic!("lex() did not yield an error.")
            };
            self
        }
    }
}

