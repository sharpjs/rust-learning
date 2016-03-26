// Lexical Analyzer
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

use std::collections::HashMap;
use std::mem;

use aex::Compilation;
use aex::mem::interner::StringInterner;
use aex::message::Messages;
use aex::pos::Pos;

// -----------------------------------------------------------------------------
// Tokens

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Token<'a> {
    Raw  (&'a str),     // Raw output
    Id   (&'a str),     // Identifier
    Flag (&'a str),     // Condition flag

    Int  (u64),         // Literal: integer
    Char (char),        // Literal: character
    Str  (&'a str),     // Literal: string

    KwType,             // Keyword: type
    KwStruct,           // Keyword: struct
    KwUnion,            // Keyword: union
    KwIf,               // Keyword: if
    KwElse,             // Keyword: else
    KwLoop,             // Keyword: loop
    KwWhile,            // Keyword: while
    KwBreak,            // Keyword: break
    KwContinue,         // Keyword: continue
    KwReturn,           // Keyword: return
    KwJump,             // Keyword: jump

    BraceL,             // {
    BraceR,             // }
    ParenL,             // (
    ParenR,             // )
    BracketL,           // [
    BracketR,           // ]
    Dot,                // .
    At,                 // @
    PlusPlus,           // ++
    MinusMinus,         // --
    Bang,               // !
    Tilde,              // ~
    Star,               // *
    Slash,              // /
    Percent,            // %
    Plus,               // +
    Minus,              // -
    LessLess,           // <<
    MoreMore,           // >>
    Ampersand,          // &
    Caret,              // ^
    Pipe,               // |
    DotTilde,           // .~
    DotBang,            // .!
    DotEqual,           // .=
    DotQuestion,        // .?
    Question,           // ?
    LessMore,           // <>
    EqualEqual,         // ==
    BangEqual,          // !=
    Less,               // <
    More,               // >
    LessEqual,          // <=
    MoreEqual,          // >=
    EqualArrow,         // =>
    MinusArrow,         // ->
    Equal,              // =
    Colon,              // :
    Comma,              // ,

    Eos,                // End of statement
    Eof,                // End of file

    Error               // Lexical error
}
use self::Token::*;

// -----------------------------------------------------------------------------
// State

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum State {
    Initial, InSpace, AfterEos,
    InIdOrKw,
    AfterZero, InNumDec, InNumHex, InNumOct, InNumBin,
    InChar, AtCharEnd, InStr, InRaw,
    InEsc, AtEscHex0, AtEscHex1, AtEscUni0, AtEscUni1,
    AfterDot, AfterPlus, AfterMinus,
    AfterLess, AfterMore,  AfterEqual, AfterBang,
    AtEof
}
use self::State::*;

// -----------------------------------------------------------------------------
// Action Codes

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Action {
    Start,              //
    Skip,               //
    SkipEol,            //

    YieldEos,           //
    YieldEosEol,        //
    YieldEof,           //

    AccumNumDec,        //
    AccumNumHexDig,     //
    AccumNumHexUc,      //
    AccumNumHexLc,      //
    AccumNumOct,        //
    AccumNumBin,        //
    YieldNum,           //

    AccumStr,           //
    YieldChar,          //
    YieldStr,           //
    YieldRaw,           //
    YieldIdOrKw,        //

    StartEsc,           //
    YieldEscNul,        //
    YieldEscLf,         //
    YieldEscCr,         //
    YieldEscTab,        //
    YieldEscChar,       //
    YieldEscNum,        //
    YieldEscHexDig,     //
    YieldEscHexUc,      //
    YieldEscHexLc,      //

    YieldBraceL,        //
    YieldBraceR,        //
    YieldParenL,        //
    YieldParenR,        //
    YieldBracketL,      //
    YieldBracketR,      //
    YieldDot,           //
    YieldAt,            //
    YieldPlusPlus,      //
    YieldMinusMinus,    //
    YieldBang,          //
    YieldBang2,         //
    YieldTilde,         //
    YieldQuestion,      //
    YieldStar,          //
    YieldSlash,         //
    YieldPercent,       //
    YieldPlus,          //
    YieldMinus,         //
    YieldLessLess,      //
    YieldMoreMore,      //
    YieldAmpersand,     //
    YieldCaret,         //
    YieldPipe,          //
    YieldDotTilde,      //
    YieldDotBang,       //
    YieldDotEqual,      //
    YieldDotQuestion,   //
    YieldLessMore,      //
    YieldEqualEqual,    //
    YieldBangEqual,     //
    YieldLess,          //
    YieldMore,          //
    YieldLessEqual,     //
    YieldMoreEqual,     //
    YieldEqualArrow,    //
    YieldMinusArrow,    //
    YieldEqual,         //
    YieldColon,         //
    YieldComma,         //

    ErrorInvalid,       //
    ErrorInvalidNum,    //
    ErrorInvalidEsc,    //
    ErrorUntermChar,    //
    ErrorUntermStr,     //
    ErrorUntermRaw,     //
    ErrorUntermEsc,     //
    ErrorLengthChar,    //
}
use self::Action::*;

// -----------------------------------------------------------------------------
// Lexer

pub struct Lexer<'a, I>
where I: Iterator<Item=char>
{
    iter:       I,                      // remaining chars
    ch:         Option<char>,           // char  after previous token
    state:      State,                  // state after previous token
    context:    Context<'a>             // context object give to actions
}

impl<'a, I> Lexer<'a, I>
where I: Iterator<Item=char>
{
    pub fn new(compilation: &'a mut Compilation<'a>, mut iter: I) -> Self {
        let ch = iter.next();

        Lexer {
            iter:    iter,
            ch:      ch,
            state:   match ch { Some(_) => Initial, None => AtEof },
            context: Context::new(compilation)
        }
    }

    pub fn strings(&self) -> &StringInterner<'a> {
        &self.context.strings
    }

    pub fn lex(&mut self) -> (Pos<'a>, Token<'a>, Pos<'a>) {
        let mut ch    =      self.ch;
        let mut state =      self.state;
        let     iter  = &mut self.iter;
        let     l     = &mut self.context;

        println!("\nlex: state = {:?}", state);

        loop {
            // Lookup handler for this state and char
            let (c, (next_state, action)) = lookup(&STATES[state as usize], ch);
            println!("lex: {:?} {:?} => {:?} {:?}", state, c, next_state, action);
            state = next_state;

            // Action code helpers
            macro_rules! consume {
                () => {{
                    l.current.byte   += c.len_utf8();
                    l.current.column += 1;
                    ch                = iter.next();
                }};
            }
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
                Start               => { l.start();               continue },
                Skip                => { consume!();              continue },
                SkipEol             => { consume!(); l.newline(); continue },

                // Terminators
                YieldEos            => { consume!();              Eos },
                YieldEosEol         => { consume!(); l.newline(); Eos },
                YieldEof            => {                          Eof },

                // Numbers
                AccumNumDec         => { consume!(); maybe!(l.num_add_dec     (c)); continue },
                AccumNumHexDig      => { consume!(); maybe!(l.num_add_hex_dig (c)); continue },
                AccumNumHexUc       => { consume!(); maybe!(l.num_add_hex_uc  (c)); continue },
                AccumNumHexLc       => { consume!(); maybe!(l.num_add_hex_lc  (c)); continue },
                AccumNumOct         => { consume!(); maybe!(l.num_add_oct     (c)); continue },
                AccumNumBin         => { consume!(); maybe!(l.num_add_bin     (c)); continue },

                YieldNum            => { l.num_get() },

                // Identifiers & Keywords
                AccumStr            => { consume!(); l.str_add(c); continue },
                YieldChar           => { consume!(); l.str_get_char() },
                YieldStr            => { consume!(); l.str_get_str()  },
                YieldRaw            => { consume!(); l.str_get_raw()  },
                YieldIdOrKw         => { l.str_get_id_or_keyword() },

                // Strings & Chars
                StartEsc            => { consume!(); push!(InEsc);            continue },
                YieldEscNul         => { consume!(); pop!(); l.str_add('\0'); continue },
                YieldEscLf          => { consume!(); pop!(); l.str_add('\n'); continue },
                YieldEscCr          => { consume!(); pop!(); l.str_add('\r'); continue },
                YieldEscTab         => { consume!(); pop!(); l.str_add('\t'); continue },
                YieldEscChar        => { consume!(); pop!(); l.str_add(  c ); continue },
                YieldEscNum         => { consume!(); pop!(); maybe!(l.str_add_esc());          continue },
                YieldEscHexDig      => { consume!(); pop!(); maybe!(l.str_add_esc_hex_dig(c)); continue },
                YieldEscHexUc       => { consume!(); pop!(); maybe!(l.str_add_esc_hex_uc(c));  continue },
                YieldEscHexLc       => { consume!(); pop!(); maybe!(l.str_add_esc_hex_lc(c));  continue },

                // Simple Tokens
                YieldBraceL         => { consume!(); BraceL      },
                YieldBraceR         => { consume!(); BraceR      },
                YieldParenL         => { consume!(); ParenL      },
                YieldParenR         => { consume!(); ParenR      },
                YieldBracketL       => { consume!(); BracketL    },
                YieldBracketR       => { consume!(); BracketR    },
                YieldDot            => {             Dot         },
                YieldAt             => { consume!(); At          },
                YieldPlusPlus       => { consume!(); PlusPlus    },
                YieldMinusMinus     => { consume!(); MinusMinus  },
                YieldBang           => {             Bang        },
                YieldBang2          => { consume!(); Bang        },
                YieldTilde          => { consume!(); Tilde       },
                YieldQuestion       => { consume!(); Question    },
                YieldStar           => { consume!(); Star        },
                YieldSlash          => { consume!(); Slash       },
                YieldPercent        => { consume!(); Percent     },
                YieldPlus           => {             Plus        },
                YieldMinus          => {             Minus       },
                YieldLessLess       => { consume!(); LessLess    },
                YieldMoreMore       => { consume!(); MoreMore    },
                YieldAmpersand      => { consume!(); Ampersand   },
                YieldCaret          => { consume!(); Caret       },
                YieldPipe           => { consume!(); Pipe        },
                YieldDotTilde       => { consume!(); DotTilde    },
                YieldDotBang        => { consume!(); DotBang     },
                YieldDotEqual       => { consume!(); DotEqual    },
                YieldDotQuestion    => { consume!(); DotQuestion },
                YieldLessMore       => { consume!(); LessMore    },
                YieldEqualEqual     => { consume!(); EqualEqual  },
                YieldBangEqual      => { consume!(); BangEqual   },
                YieldLess           => {             Less        },
                YieldMore           => {             More        },
                YieldLessEqual      => { consume!(); LessEqual   },
                YieldMoreEqual      => { consume!(); MoreEqual   },
                YieldEqualArrow     => { consume!(); EqualArrow  },
                YieldMinusArrow     => { consume!(); MinusArrow  },
                YieldEqual          => {             Equal       },
                YieldColon          => { consume!(); Colon       },
                YieldComma          => { consume!(); Comma       },

                // Errors
                ErrorInvalid        => { l.messages.err_unrec       (l.start, c); Error },
                ErrorInvalidNum     => { l.messages.err_unrec_num   (l.start, c); Error },
                ErrorInvalidEsc     => { l.messages.err_unrec_esc   (l.start, c); Error },
                ErrorUntermChar     => { l.messages.err_unterm_char (l.start,  ); Error },
                ErrorUntermStr      => { l.messages.err_unterm_str  (l.start,  ); Error },
                ErrorUntermRaw      => { l.messages.err_unterm_raw  (l.start,  ); Error },
                ErrorUntermEsc      => { l.messages.err_unterm_esc  (l.start,  ); Error },
                ErrorLengthChar     => { l.messages.err_length_char (l.start,  ); Error },
            };

            // Remember state for next invocation
            self.ch    = ch;
            self.state = state;

            // Yield
            let start = l.start; l.start();
            let end   = l.current;
            println!("lex: yield {:?}", (start, token, end));
            return (start, token, end);
        }
    }
}

// -----------------------------------------------------------------------------
// State Transition Table

type TransitionSet = (
    [u8; 128],          // Map from 7-bit char to transition index
    &'static [(         // Array of transitions:
        State,          //   - next state
        Action          //   - custom action
    )]
);

#[inline]
fn lookup(entry: &TransitionSet, ch: Option<char>) -> (char, (State, Action))
{
    // Lookup A: char -> transition index
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

    // Lookup B: transition index -> transition
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
        //              State       Action
        /*  0: eof */ ( AtEof,      YieldEof       ),
        /*  1: ??? */ ( AtEof,      ErrorInvalid   ),
        /*  2: \s  */ ( InSpace,    Skip           ),
        /*  3: \n  */ ( AfterEos,   YieldEosEol    ),
        /*  4:  ;  */ ( AfterEos,   YieldEos       ),
        /*  5: id0 */ ( InIdOrKw,   AccumStr       ),

        /*  6:  0  */ ( AfterZero,  Skip           ),
        /*  7: 1-9 */ ( InNumDec,   AccumNumDec    ),
        /*  8:  '  */ ( InChar,     Skip           ),
        /*  9:  "  */ ( InStr,      Skip           ),
        /* 10:  `  */ ( InRaw,      Skip           ),
        /* 11:  {  */ ( Initial,    YieldBraceL    ),
        /* 12:  }  */ ( Initial,    YieldBraceR    ),
        /* 13:  (  */ ( Initial,    YieldParenL    ),
        /* 14:  )  */ ( Initial,    YieldParenR    ),
        /* 15:  [  */ ( Initial,    YieldBracketL  ),
        /* 16:  ]  */ ( Initial,    YieldBracketR  ),
        /* 17:  .  */ ( AfterDot,   Skip           ),
        /* 18:  @  */ ( Initial,    YieldAt        ),
        /* 19:  !  */ ( AfterBang,  Skip           ),
        /* 20:  ~  */ ( Initial,    YieldTilde     ),
        /* 21:  ?  */ ( Initial,    YieldQuestion  ),
        /* 22:  *  */ ( Initial,    YieldStar      ),
        /* 23:  /  */ ( Initial,    YieldSlash     ),
        /* 24:  %  */ ( Initial,    YieldPercent   ),
        /* 25:  +  */ ( AfterPlus,  Skip           ),
        /* 26:  -  */ ( AfterMinus, Skip           ),
        /* 27:  &  */ ( Initial,    YieldAmpersand ),
        /* 28:  ^  */ ( Initial,    YieldCaret     ),
        /* 29:  |  */ ( Initial,    YieldPipe      ),
        /* 30:  <  */ ( AfterLess,  Skip           ),
        /* 31:  >  */ ( AfterMore,  Skip           ),
        /* 32:  =  */ ( AfterEqual, Skip           ),
        /* 33:  :  */ ( Initial,    YieldColon     ),
        /* 34:  ,  */ ( Initial,    YieldComma     ),
    ]),

    // InSpace - In whitespace
    ([
        x, x, x, x, x, x, x, x,  x, 2, x, x, x, 2, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        2, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // 01234567 89:;<=>?
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             State    Action
        /* 0: eof */ ( AtEof,   Start ),
        /* 1: ??? */ ( Initial, Start ),
        /* 2: \s  */ ( InSpace, Skip  ),
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
        //             State     Action
        /* 0: eof */ ( AtEof,    Start   ),
        /* 1: ??? */ ( Initial,  Start   ),
        /* 2: \s; */ ( AfterEos, Skip    ),
        /* 3: \n  */ ( AfterEos, SkipEol ),
    ]),

    // InIdOrKw - In identifier or keyword
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
        //             State     Action
        /* 0: eof */ ( AtEof,    YieldIdOrKw ),
        /* 1: ??? */ ( Initial,  YieldIdOrKw ),
        /* 2: id  */ ( InIdOrKw, AccumStr    ),
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
        //             State     Action
        /* 0: eof */ ( AtEof,    YieldNum        ),
        /* 1: ??? */ ( AtEof,    ErrorInvalidNum ),
        /* 2: 0-9 */ ( InNumDec, AccumNumDec     ),
        /* 3:  _  */ ( InNumDec, Skip            ),
        /* 4:  x  */ ( InNumHex, Skip            ),
        /* 5:  o  */ ( InNumOct, Skip            ),
        /* 6:  b  */ ( InNumBin, Skip            ),
        /* 7: \s  */ ( InSpace,  YieldNum        ),
        /* 8: opr */ ( Initial,  YieldNum        ),
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
        //             State     Action
        /* 0: eof */ ( AtEof,    YieldNum        ),
        /* 1: ??? */ ( AtEof,    ErrorInvalidNum ),
        /* 2: 0-9 */ ( InNumDec, AccumNumDec     ),
        /* 3:  _  */ ( InNumDec, Skip            ),
        /* 4: \s  */ ( InSpace,  YieldNum        ),
        /* 5: opr */ ( Initial,  YieldNum        ),
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
        //             State     Action
        /* 0: eof */ ( AtEof,    YieldNum        ),
        /* 1: ??? */ ( AtEof,    ErrorInvalidNum ),
        /* 2: 0-9 */ ( InNumHex, AccumNumHexDig  ),
        /* 3: A-F */ ( InNumHex, AccumNumHexUc   ),
        /* 4: a-f */ ( InNumHex, AccumNumHexLc   ),
        /* 5:  _  */ ( InNumHex, Skip            ),
        /* 6: \s  */ ( InSpace,  YieldNum        ),
        /* 7: opr */ ( Initial,  YieldNum        ),
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
        //             State     Action
        /* 0: eof */ ( AtEof,    YieldNum        ),
        /* 1: ??? */ ( AtEof,    ErrorInvalidNum ),
        /* 2: 0-7 */ ( InNumOct, AccumNumOct     ),
        /* 3:  _  */ ( InNumOct, Skip            ),
        /* 6: \s  */ ( InSpace,  YieldNum        ),
        /* 4: opr */ ( Initial,  YieldNum        ),
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
        //             State     Action
        /* 0: eof */ ( AtEof,    YieldNum        ),
        /* 1: ??? */ ( AtEof,    ErrorInvalidNum ),
        /* 2: 0-1 */ ( InNumBin, AccumNumBin     ),
        /* 3:  _  */ ( InNumBin, Skip            ),
        /* 6: \s  */ ( InSpace,  YieldNum        ),
        /* 4: opr */ ( Initial,  YieldNum        ),
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
        //             State      Action
        /* 0: eof */ ( AtEof,     ErrorUntermChar ),
        /* 1: ??? */ ( AtCharEnd, AccumStr        ),
        /* 2:  \  */ ( AtCharEnd, StartEsc        ),
        /* 3:  '  */ ( AtEof,     ErrorLengthChar ),
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
        //             State    Action
        /* 0: eof */ ( AtEof,   ErrorUntermChar ),
        /* 1: ??? */ ( AtEof,   ErrorLengthChar ),
        /* 2:  '  */ ( Initial, YieldChar       ),
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
        //             State    Action
        /* 0: eof */ ( AtEof,   ErrorUntermStr ),
        /* 1: ??? */ ( InStr,   AccumStr       ),
        /* 2:  \  */ ( InStr,   StartEsc       ),
        /* 3:  "  */ ( Initial, YieldStr       ),
    ]),

    // InRaw <( ` )> : in a raw output block
    ([
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ .tn..r..
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // 01234567 89:;<=>?
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
        2, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             State    Action
        /* 0: eof */ ( AtEof,   ErrorUntermRaw ),
        /* 1: ??? */ ( InRaw,   AccumStr       ),
        /* 3:  `  */ ( Initial, YieldRaw       ),
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
        //              State      Action
        /*  0: eof */ ( AtEof,     ErrorUntermEsc  ),
        /*  1: ??? */ ( AtEof,     ErrorInvalidEsc ),
        /*  2:  0  */ ( InStr,     YieldEscNul     ), // pops state
        /*  3:  n  */ ( InStr,     YieldEscLf      ), // pops state
        /*  4:  r  */ ( InStr,     YieldEscCr      ), // pops state
        /*  5:  t  */ ( InStr,     YieldEscTab     ), // pops state
        /*  6:  \  */ ( InStr,     YieldEscChar    ), // pops state
        /*  7:  '  */ ( InStr,     YieldEscChar    ), // pops state
        /*  8:  "  */ ( InStr,     YieldEscChar    ), // pops state
        /*  9:  x  */ ( AtEscHex0, Skip            ),
        /* 10:  u  */ ( AtEscUni0, Skip            ),
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
        //             State      Action
        /* 0: eof */ ( AtEof,     ErrorUntermEsc  ),
        /* 1: ??? */ ( AtEof,     ErrorInvalidEsc ),
        /* 2: 0-9 */ ( AtEscHex1, AccumNumHexDig  ),
        /* 3: A-F */ ( AtEscHex1, AccumNumHexUc   ),
        /* 4: a-f */ ( AtEscHex1, AccumNumHexLc   ),
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
        //             State  Action
        /* 0: eof */ ( AtEof, ErrorUntermEsc  ),
        /* 1: ??? */ ( AtEof, ErrorInvalidEsc ),
        /* 2: 0-9 */ ( InStr, YieldEscHexDig  ), // pops state
        /* 3: A-F */ ( InStr, YieldEscHexUc   ), // pops state
        /* 4: a-f */ ( InStr, YieldEscHexLc   ), // pops state
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
        //             State      Action
        /* 0: eof */ ( AtEof,     ErrorUntermEsc  ),
        /* 1: ??? */ ( AtEof,     ErrorInvalidEsc ),
        /* 2:  {  */ ( AtEscUni1, Skip            ),
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
        //             State      Action
        /* 0: eof */ ( AtEof,     ErrorUntermEsc  ),
        /* 1: ??? */ ( AtEof,     ErrorInvalidEsc ),
        /* 2: 0-9 */ ( AtEscUni1, AccumNumHexDig  ),
        /* 3: A-F */ ( AtEscUni1, AccumNumHexUc   ),
        /* 4: a-f */ ( AtEscUni1, AccumNumHexLc   ),
        /* 5:  }  */ ( InStr,     YieldEscNum     ), // pops state
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
        //             State    Action
        /* 0: eof */ ( AtEof,   YieldDot         ),
        /* 1: ??? */ ( Initial, YieldDot         ),
        /* 2: \s  */ ( Initial, YieldDot         ),
        /* 3:  ~  */ ( Initial, YieldDotTilde    ),
        /* 4:  !  */ ( Initial, YieldDotBang     ),
        /* 5:  =  */ ( Initial, YieldDotEqual    ),
        /* 6:  ?  */ ( Initial, YieldDotQuestion ),
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
        //             State    Action
        /* 0: eof */ ( AtEof,   YieldPlus     ),
        /* 1: ??? */ ( Initial, YieldPlus     ),
        /* 2: \s  */ ( Initial, YieldPlus     ),
        /* 3:  +  */ ( Initial, YieldPlusPlus ),
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
        //             State    Action
        /* 0: eof */ ( AtEof,   YieldMinus      ),
        /* 1: ??? */ ( Initial, YieldMinus      ),
        /* 2: \s  */ ( Initial, YieldMinus      ),
        /* 3:  -  */ ( Initial, YieldMinusMinus ),
        /* 4:  >  */ ( Initial, YieldMinusArrow ),
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
        //             State    Action
        /* 0: eof */ ( AtEof,   YieldLess      ),
        /* 1: ??? */ ( Initial, YieldLess      ),
        /* 2: \s  */ ( Initial, YieldLess      ),
        /* 3:  <  */ ( Initial, YieldLessLess  ),
        /* 4:  >  */ ( Initial, YieldLessMore  ),
        /* 5:  =  */ ( Initial, YieldLessEqual ),
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
        //             State    Action
        /* 0: eof */ ( AtEof,   YieldMore      ),
        /* 1: ??? */ ( Initial, YieldMore      ),
        /* 2: \s  */ ( Initial, YieldMore      ),
        /* 3:  <  */ ( Initial, YieldMoreMore  ), // TODO: exchange
        /* 4:  >  */ ( Initial, YieldMoreMore  ),
        /* 5:  =  */ ( Initial, YieldMoreEqual ),
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
        //             State    Action
        /* 0: eof */ ( AtEof,   YieldEqual      ),
        /* 1: ??? */ ( Initial, YieldEqual      ),
        /* 2: \s  */ ( Initial, YieldEqual      ),
        /* 3:  >  */ ( Initial, YieldEqualArrow ),
        /* 4:  =  */ ( Initial, YieldEqualEqual ),
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
        //             State      Action
        /* 0: eof */ ( AtEof,     YieldBang      ),
        /* 1: ??? */ ( Initial,   YieldBang      ),
        /* 2: \s  */ ( Initial,   YieldBang      ),
        /* 3:  =  */ ( Initial,   YieldBangEqual ),
        /* 4:  !  */ ( AfterBang, YieldBang2     ), // need to consume!
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
//      //             State  Next?  Action
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
        //             State  Action
        /* 0: eof */ ( AtEof, YieldEof ),
    ]),
];

// -----------------------------------------------------------------------------
// Keywords

const KEYWORDS: &'static [(&'static str, Token<'static>)] = &[
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

struct Context<'a> {
    start:      Pos<'a>,                        // position of token start
    current:    Pos<'a>,                        // position of current character
    number:     u64,                            // number builder
    buffer:     String,                         // string builder
    strings:    &'a StringInterner<'a>,         // string interner
    keywords:   HashMap<&'a str, Token<'a>>,    // keyword table
    messages:   &'a mut Messages<'a>            // messages collector
}

impl<'a> Context<'a> {
    fn new(compilation: &'a mut Compilation<'a>) -> Self {
        let strings = &compilation.strings;

        let mut keywords = HashMap::new();
        for &(k, t) in KEYWORDS {
            keywords.insert(strings.intern_ref(k), t);
        }

        Context {
            start:    Pos { file: "", byte: 0, line: 1, column: 1 },
            current:  Pos { file: "", byte: 0, line: 1, column: 1 },
            buffer:   String::with_capacity(128),
            number:   0,
            strings:  strings,
            keywords: keywords,
            messages: &mut compilation.log
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
    fn num_add_dec(&mut self, c: char) -> Option<Token<'a>> {
        self.num_add(10, int_from_dig(c))
    }

    #[inline]
    fn num_add_hex_dig(&mut self, c: char) -> Option<Token<'a>> {
        self.num_add(16, int_from_dig(c))
    }

    #[inline]
    fn num_add_hex_uc(&mut self, c: char) -> Option<Token<'a>> {
        self.num_add(16, int_from_hex_uc(c))
    }

    #[inline]
    fn num_add_hex_lc(&mut self, c: char) -> Option<Token<'a>> {
        self.num_add(16, int_from_hex_lc(c))
    }

    #[inline]
    fn num_add_oct(&mut self, c: char) -> Option<Token<'a>> {
        self.num_add(8, int_from_dig(c))
    }

    #[inline]
    fn num_add_bin(&mut self, c: char) -> Option<Token<'a>> {
        self.num_add(2, int_from_dig(c))
    }

    #[inline]
    fn num_add(&mut self, base: u8, digit: u8) -> Option<Token<'a>> {
        let mut n = self.number;

        n = match n.checked_mul(base as u64) {
            Some(n) => n,
            None    => return self.err_overflow_num()
        };

        n = match n.checked_add(digit as u64) {
            Some(n) => n,
            None    => return self.err_overflow_num()
        };

        self.number = n;
        None
    }

    #[inline]
    fn num_get(&mut self) -> Token<'a> {
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
    fn str_add_esc(&mut self) -> Option<Token<'a>> {
        let n = self.number as u32;
        if  n > UNICODE_MAX { return self.err_overflow_esc() }
        let c = unsafe { mem::transmute(n) };
        self.buffer.push(c);
        None
    }

    #[inline]
    fn str_add_esc_hex_dig(&mut self, c: char) -> Option<Token<'a>> {
        self.num_add_hex_dig(c)
            .or_else(|| self.str_add_esc())
    }

    #[inline]
    fn str_add_esc_hex_uc(&mut self, c: char) -> Option<Token<'a>> {
        self.num_add_hex_uc(c)
            .or_else(|| self.str_add_esc())
    }

    #[inline]
    fn str_add_esc_hex_lc(&mut self, c: char) -> Option<Token<'a>> {
        self.num_add_hex_lc(c)
            .or_else(|| self.str_add_esc())
    }

    #[inline]
    fn str_intern(&mut self) -> &'a str {
        let id = self.strings.intern(self.buffer.clone());
        self.buffer.clear();
        id
    }

    #[inline]
    fn str_get_char(&mut self) -> Token<'a> {
        let c = self.buffer.chars().next().unwrap();
        self.buffer.clear();
        Char(c)
    }

    #[inline]
    fn str_get_str(&mut self) -> Token<'a> {
        Str(self.str_intern())
    }

    #[inline]
    fn str_get_raw(&mut self) -> Token<'a> {
        Raw(self.str_intern())
    }

    #[inline]
    fn str_get_id_or_keyword(&mut self) -> Token<'a> {
        let id = self.str_intern();

        match self.keywords.get(&id) {
            Some(&k) => k,
            None     => Id(id)
        }
    }

    // Error Actions

    fn err_overflow_num(&mut self) -> Option<Token<'a>> {
        self.messages.err_overflow_num(self.start);
        Some(Error)
    }

    fn err_overflow_esc(&mut self) -> Option<Token<'a>> {
        self.messages.err_overflow_esc(self.start);
        Some(Error)
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
        lex("", |it| { it.yields(Eof); });
    }

    #[test]
    fn space() {
        lex( " \r\t" , |it| { it                              .yields(Eof); });
        lex( " \r\t1", |it| { it.yields(Int(1))               .yields(Eof); });
        lex("1 \r\t" , |it| { it.yields(Int(1))               .yields(Eof); });
        lex("1 \r\t2", |it| { it.yields(Int(1)).yields(Int(2)).yields(Eof); });
    }

    #[test]
    fn eos() {
        lex(";"         , |it| { it.yields(Eos)               .yields(Eof); });
        lex("\n"        , |it| { it.yields(Eos)               .yields(Eof); });
        lex(";1"        , |it| { it.yields(Eos).yields(Int(1)).yields(Eof); });
        lex("\n1"       , |it| { it.yields(Eos).yields(Int(1)).yields(Eof); });
        lex("; \r\t\n;" , |it| { it.yields(Eos)               .yields(Eof); });
        lex("\n \r\t\n;", |it| { it.yields(Eos)               .yields(Eof); });
    }

    #[test]
    fn id() {
        lex("a"                         , |it| { it.yields(Id("a"))                         .yields(Eof); });
        lex("a "                        , |it| { it.yields(Id("a"))                         .yields(Eof); });
        lex("abcdefghijklmnopqrstuvwxyz", |it| { it.yields(Id("abcdefghijklmnopqrstuvwxyz")).yields(Eof); });
        lex("ABCDEFGHIJKLMNOPQRSTUVWXYZ", |it| { it.yields(Id("ABCDEFGHIJKLMNOPQRSTUVWXYZ")).yields(Eof); });
        lex("_0123456789"               , |it| { it.yields(Id("_0123456789"))               .yields(Eof); });
    }

    #[test]
    fn num() {
        lex( "123456789", |it| { it.yields(Int( 123456789)).yields(Eof); });
        lex("0x01234567", |it| { it.yields(Int(0x01234567)).yields(Eof); });
        lex("0x89ABCDEF", |it| { it.yields(Int(0x89ABCDEF)).yields(Eof); });
        lex("0x89abcdef", |it| { it.yields(Int(0x89ABCDEF)).yields(Eof); });
        lex("0o01234567", |it| { it.yields(Int(0o01234567)).yields(Eof); });
        lex(      "0b01", |it| { it.yields(Int(      0b01)).yields(Eof); });
        lex(       "012", |it| { it.yields(Int(        12)).yields(Eof); });
        lex(         "0", |it| { it.yields(Int(         0)).yields(Eof); });
        lex(    "1__2__", |it| { it.yields(Int(        12)).yields(Eof); });
        lex("0x__1__2__", |it| { it.yields(Int(      0x12)).yields(Eof); });
        lex(       "0__", |it| { it.yields(Int(         0)).yields(Eof); });
        lex(     "0b1  ", |it| { it.yields(Int(         1)).yields(Eof); });
        lex(      "0b1;", |it| { it.yields(Int(         1)).yields(Eos).yields(Eof); });
        lex(     "0b19z", |it| { it.yields_error(); });
    }

    #[test]
    fn char() {
        lex("'a'" , |it| { it.yields(Char('a')).yields(Eof); });
        lex("'a"  , |it| { it.yields_error(); });
        lex("''"  , |it| { it.yields_error(); });
        lex("'aa'", |it| { it.yields_error(); });
    }

    #[test]
    fn str() {
        lex("\"\""  , |it| { it.yields(Str(""  )).yields(Eof); });
        lex("\"aa\"", |it| { it.yields(Str("aa")).yields(Eof); });
        lex("\"a"   , |it| { it.yields_error(); });
    }

    #[test]
    fn char_escape() {
        lex("'\\0'" , |it| { it.yields(Char('\0')).yields(Eof); });
        lex("'\\n'" , |it| { it.yields(Char('\n')).yields(Eof); });
        lex("'\\r'" , |it| { it.yields(Char('\r')).yields(Eof); });
        lex("'\\t'" , |it| { it.yields(Char('\t')).yields(Eof); });
        lex("'\\\\'", |it| { it.yields(Char('\\')).yields(Eof); });
        lex("'\\\''", |it| { it.yields(Char('\'')).yields(Eof); });
        lex("'\\\"'", |it| { it.yields(Char('\"')).yields(Eof); });
        lex("'\\a'" , |it| { it.yields_error(); });
    }

    #[test]
    fn char_escape_hex() {
        lex("'\\x5A'" , |it| { it.yields(Char('\u{5A}')).yields(Eof); });
        lex("'\\xA5'" , |it| { it.yields(Char('\u{A5}')).yields(Eof); }); // TODO: Is this a byte?
        lex("'\\x"    , |it| { it.yields_error(); });
        lex("'\\x'"   , |it| { it.yields_error(); });
        lex("'\\x5"   , |it| { it.yields_error(); });
        lex("'\\x5'"  , |it| { it.yields_error(); });
        lex("'\\x5Ax'", |it| { it.yields_error(); });
    }

    #[test]
    fn char_escape_uni() {
        lex("'\\u"       , |it| { it.yields_error(); });
        lex("'\\u'"      , |it| { it.yields_error(); });
        lex("'\\u{"      , |it| { it.yields_error(); });
        lex("'\\u{'"     , |it| { it.yields_error(); });
        lex("'\\u{}'"    , |it| { it.yields(Char('\u{000}')).yields(Eof); });
        lex("'\\u{1Fe}'" , |it| { it.yields(Char('\u{1Fe}')).yields(Eof); });
        lex("'\\u{1Fe}x'", |it| { it.yields_error(); });
    }

    #[test]
    fn keywords() {
        lex("type"    , |it| { it.yields(KwType    ).yields(Eof); });
        lex("struct"  , |it| { it.yields(KwStruct  ).yields(Eof); });
        lex("union"   , |it| { it.yields(KwUnion   ).yields(Eof); });
        lex("if"      , |it| { it.yields(KwIf      ).yields(Eof); });
        lex("else"    , |it| { it.yields(KwElse    ).yields(Eof); });
        lex("loop"    , |it| { it.yields(KwLoop    ).yields(Eof); });
        lex("while"   , |it| { it.yields(KwWhile   ).yields(Eof); });
        lex("break"   , |it| { it.yields(KwBreak   ).yields(Eof); });
        lex("continue", |it| { it.yields(KwContinue).yields(Eof); });
        lex("return"  , |it| { it.yields(KwReturn  ).yields(Eof); });
        lex("jump"    , |it| { it.yields(KwJump    ).yields(Eof); });
    }

    #[test]
    fn raw() {
        lex("`a`", |it| { it.yields(Raw("a")); });
    }

    #[test]
    fn punctuation() {
        lex("{ } ( ) [ ] . @ ++ -- ! ~ ? * / % + - << >> & ^ | .~ .! .= .? \
             <> == != < > <= >= => -> = : ,", |it| { it
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
        });
    }

    // Test Harness

    use std::str::Chars;
    use aex::mem::interner::*;

    fn lex<'a, F>(input: &'a str, assert: F)
                 where F: FnOnce(&mut LexerHarness)
    {
        let     strings = StringInterner::new();
        let     chars   = input.chars();
        let     lexer   = Lexer::new(&strings, chars);
        let mut harness = LexerHarness(lexer);
        assert(&mut harness)
    }

    struct LexerHarness<'a> (Lexer<'a, Chars<'a>>);

    impl<'a> LexerHarness<'a> {
        fn yields(&mut self, token: Token) -> &mut Self {
            assert_eq!(token, self.0.lex().1);
            self
        }

        fn yields_error(&mut self) -> &mut Self {
            self.yields(Error)
        }
    }
}

