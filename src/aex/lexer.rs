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

use std::str::Chars;

//use aex::compiler::Compiler;
use aex::source::{File, Source};
use aex::token::{Token, TokenBuilder, Compiler};
use aex::token::Token::*;

// -----------------------------------------------------------------------------
// State

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum State {
    Initial, InSpace, AfterEos, InIdOrKw,
    AfterZero, InNumDec, InNumHex, InNumOct, InNumBin,
    InChar, AtCharEnd, InStr,
    InEsc, AtEscHex0, AtEscHex1, AtEscUni0, AtEscUni1,
  //InOp,
  //AfterDot, AfterPlus, AfterMinus,
  //AfterLess, AfterMore,  AfterEqual, AfterBang,
    AtEof
}
use self::State::*;

// -----------------------------------------------------------------------------
// Action Codes

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Action {
    Skip,               //
    SkipEol,            //
    Start,              //

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
    YieldIdOrKw,        //

    StartEsc,           //
    AccumEscNul,        //
    AccumEscLf,         //
    AccumEscCr,         //
    AccumEscTab,        //
    AccumEscChar,       //
    AccumEscNum,        //
    AccumEscHexDig,     //
    AccumEscHexUc,      //
    AccumEscHexLc,      //

    YieldBraceL,        //
    YieldBraceR,        //
    YieldParenL,        //
    YieldParenR,        //
    YieldBracketL,      //
    YieldBracketR,      //
  //YieldDot,           //
  //YieldAt,            //
  //YieldEqualArrow,    //
  //YieldMinusArrow,    //
  //YieldEqual,         //
  //YieldColon,         //
  //YieldComma,         //
  //YieldOperator,      //

    ErrorInvalid,       //
    ErrorInvalidNum,    //
    ErrorInvalidEsc,    //
    ErrorUntermChar,    //
    ErrorUntermStr,     //
    ErrorUntermEsc,     //
    ErrorLengthChar,    //
}
use self::Action::*;

// -----------------------------------------------------------------------------
// Lexer

pub struct Lexer<'a>
{
    chars:   Chars<'a>,                 // remaining chars
    ch:      Option<char>,              // char  after previous token
    state:   State,                     // state after previous token
    builder: TokenBuilder<'a>           // used by actions to build tokens
}

impl<'a> Lexer<'a>
{
    pub fn new(compiler:  &'a Compiler <'a>,
               file:      &'a File     <'a>
              ) -> Self {
        let mut chars = file.data().chars();
        let ch        = chars.next();
        Lexer {
            chars:   chars,
            ch:      ch,
            state:   match ch { Some(_) => Initial, None => AtEof },
            builder: TokenBuilder::new(compiler, file),
        }
    }
}

pub trait Lex<'a> {
    fn lex(&mut self) -> (Source<'a>, Token<'a>);
}

impl<'a> Lex<'a> for Lexer<'a>
{
    fn lex(&mut self) -> (Source<'a>, Token<'a>) {
        let mut ch    =      self.ch;
        let mut state =      self.state;
        let     chars = &mut self.chars;
        let     t     = &mut self.builder;

        println!("\nlex: state = {:?}", state);

        loop {
            // Lookup handler for this state and char
            let (c, (next_state, action)) = lookup(&STATES[state as usize], ch);
            println!("lex: {:?} {:?} => {:?} {:?}", state, c, next_state, action);
            state = next_state;

            // Action code helpers
            macro_rules! consume {
                () => {{ t.advance(c); ch = chars.next(); }};
            }
            macro_rules! push {
                ($s:expr) => {{ self.state = state; state = $s }};
            }
            macro_rules! pop {
                () => {{ state = self.state }};
            }
            macro_rules! skip {
                ($($e:expr);*) => {{ $($e;)* continue }};
            }
            macro_rules! maybe {
                ($($e:expr);+) => {
                    match { $($e);+ } { Some(e) => e, _ => continue }
                };
            }

            // Invoke action code
            let token = match action {
                // Space
                Skip                => { consume!();              continue },
                SkipEol             => { consume!(); t.newline(); continue },
                Start               => {             t.start();   continue },

                // Terminators
                YieldEos            => { consume!();              Eos },
                YieldEosEol         => { consume!(); t.newline(); Eos },
                YieldEof            => {                          Eof },

                // Numbers
                AccumNumDec         => { consume!(); t.add_dec    (c); continue },
                AccumNumHexDig      => { consume!(); t.add_hex_dg (c); continue },
                AccumNumHexUc       => { consume!(); t.add_hex_uc (c); continue },
                AccumNumHexLc       => { consume!(); t.add_hex_lc (c); continue },
                AccumNumOct         => { consume!(); t.add_oct    (c); continue },
                AccumNumBin         => { consume!(); t.add_bin    (c); continue },

                YieldNum            => { t.get_num() },

                // Identifiers & Keywords
                AccumStr            => { consume!(); t.add_char(c); continue },
                YieldChar           => { consume!(); t.get_char()            },
                YieldStr            => { consume!(); t.get_str()             },
                YieldIdOrKw         => {             t.get_id_or_keyword()   },

                // Strings & Chars
                StartEsc            => { consume!(); push!(InEsc);             continue },
                AccumEscNul         => { consume!(); pop!(); t.add_char('\0'); continue },
                AccumEscLf          => { consume!(); pop!(); t.add_char('\n'); continue },
                AccumEscCr          => { consume!(); pop!(); t.add_char('\r'); continue },
                AccumEscTab         => { consume!(); pop!(); t.add_char('\t'); continue },
                AccumEscChar        => { consume!(); pop!(); t.add_char(  c ); continue },
                AccumEscNum         => { consume!(); pop!();                  maybe!(t.add_esc()) },
                AccumEscHexDig      => { consume!(); pop!(); t.add_hex_dg(c); maybe!(t.add_esc()) },
                AccumEscHexUc       => { consume!(); pop!(); t.add_hex_uc(c); maybe!(t.add_esc()) },
                AccumEscHexLc       => { consume!(); pop!(); t.add_hex_lc(c); maybe!(t.add_esc()) },

                // Simple Tokens
                YieldBraceL         => { consume!(); BraceL      },
                YieldBraceR         => { consume!(); BraceR      },
                YieldParenL         => { consume!(); ParenL      },
                YieldParenR         => { consume!(); ParenR      },
                YieldBracketL       => { consume!(); BracketL    },
                YieldBracketR       => { consume!(); BracketR    },
              //YieldDot            => {             Dot         },
              //YieldAt             => { consume!(); At          },
              //YieldPlusPlus       => { consume!(); PlusPlus    },
              //YieldMinusMinus     => { consume!(); MinusMinus  },
              //YieldBang           => {             Bang        },
              //YieldBang2          => { consume!(); Bang        },
              //YieldTilde          => { consume!(); Tilde       },
              //YieldQuestion       => { consume!(); Question    },
              //YieldStar           => { consume!(); Star        },
              //YieldSlash          => { consume!(); Slash       },
              //YieldPercent        => { consume!(); Percent     },
              //YieldPlus           => {             Plus        },
              //YieldMinus          => {             Minus       },
              //YieldLessLess       => { consume!(); LessLess    },
              //YieldMoreMore       => { consume!(); MoreMore    },
              //YieldAmpersand      => { consume!(); Ampersand   },
              //YieldCaret          => { consume!(); Caret       },
              //YieldPipe           => { consume!(); Pipe        },
              //YieldDotTilde       => { consume!(); DotTilde    },
              //YieldDotBang        => { consume!(); DotBang     },
              //YieldDotEqual       => { consume!(); DotEqual    },
              //YieldDotQuestion    => { consume!(); DotQuestion },
              //YieldLessMore       => { consume!(); LessMore    },
              //YieldEqualEqual     => { consume!(); EqualEqual  },
              //YieldBangEqual      => { consume!(); BangEqual   },
              //YieldLess           => {             Less        },
              //YieldMore           => {             More        },
              //YieldLessEqual      => { consume!(); LessEqual   },
              //YieldMoreEqual      => { consume!(); MoreEqual   },
              //YieldEqualArrow     => { consume!(); EqualArrow  },
              //YieldMinusArrow     => { consume!(); MinusArrow  },
              //YieldEqual          => {             Equal       },
              //YieldColon          => { consume!(); Colon       },
              //YieldComma          => { consume!(); Comma       },

                // Errors
                ErrorInvalid        => { t.err_unrec       (c) },
                ErrorInvalidNum     => { t.err_unrec_num   (c) },
                ErrorInvalidEsc     => { t.err_unrec_esc   (c) },
                ErrorUntermChar     => { t.err_unterm_char ( ) },
                ErrorUntermStr      => { t.err_unterm_str  ( ) },
                ErrorUntermEsc      => { t.err_unterm_esc  ( ) },
                ErrorLengthChar     => { t.err_length_char ( ) },
            };

            // Remember state for next invocation
            self.ch    = ch;
            self.state = state;

            // Yield
            let source = t.source();
            t.start();
            //println!("lex: yield {:?}", (start, token, end));
            return (source, token);
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
        2, x, 9, x, x, x, x, 8, 12,13, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
        6, 7, 7, 7, 7, 7, 7, 7,  7, 7, x, 4, x, x, x, x, // 01234567 89:;<=>?
        x, 5, 5, 5, 5, 5, 5, 5,  5, 5, 5, 5, 5, 5, 5, 5, // @ABCDEFG HIJKLMNO
        5, 5, 5, 5, 5, 5, 5, 5,  5, 5, 5,14, x,15, x, 5, // PQRSTUVW XYZ[\]^_
        x, 5, 5, 5, 5, 5, 5, 5,  5, 5, 5, 5, 5, 5, 5, 5, // `abcdefg hijklmno
        5, 5, 5, 5, 5, 5, 5, 5,  5, 5, 5,10, x,11, x, x, // pqrstuvw xyz{|}~. <- DEL
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
        /* 10:  {  */ ( Initial,    YieldBraceL    ),
        /* 11:  }  */ ( Initial,    YieldBraceR    ),
        /* 12:  (  */ ( Initial,    YieldParenL    ),
        /* 13:  )  */ ( Initial,    YieldParenR    ),
        /* 14:  [  */ ( Initial,    YieldBracketL  ),
        /* 15:  ]  */ ( Initial,    YieldBracketR  ),
    ]),

    //// Initial
    //([
    //    x, x, x, x, x, x, x, x,  x, 2, 3, x, x, 2, x, x, // ........ .tn..r..
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
    //    2,19, 9, x, x,24,27, 8, 13,14,22,25,34,26,17,23, //  !"#$%&' ()*+,-./
    //    6, 7, 7, 7, 7, 7, 7, 7,  7, 7,33, 4,30,32,31,21, // 01234567 89:;<=>?
    //   18, 5, 5, 5, 5, 5, 5, 5,  5, 5, 5, 5, 5, 5, 5, 5, // @ABCDEFG HIJKLMNO
    //    5, 5, 5, 5, 5, 5, 5, 5,  5, 5, 5,15, x,16,28, 5, // PQRSTUVW XYZ[\]^_
    //   10, 5, 5, 5, 5, 5, 5, 5,  5, 5, 5, 5, 5, 5, 5, 5, // `abcdefg hijklmno
    //    5, 5, 5, 5, 5, 5, 5, 5,  5, 5, 5,11,29,12,20, x, // pqrstuvw xyz{|}~. <- DEL
    //],&[
    //    //              State       Action
    //    /*  0: eof */ ( AtEof,      YieldEof       ),
    //    /*  1: ??? */ ( AtEof,      ErrorInvalid   ),
    //    /*  2: \s  */ ( InSpace,    Skip           ),
    //    /*  3: \n  */ ( AfterEos,   YieldEosEol    ),
    //    /*  4:  ;  */ ( AfterEos,   YieldEos       ),
    //    /*  5: id0 */ ( InIdOrKw,   AccumStr       ),
    //    /*  6:  0  */ ( AfterZero,  Skip           ),
    //    /*  7: 1-9 */ ( InNumDec,   AccumNumDec    ),
    //    /*  8:  '  */ ( InChar,     Skip           ),
    //    /*  9:  "  */ ( InStr,      Skip           ),
    //    /* 10:  `  */ ( AtEof,      ErrorInvalid   ), // was raw string
    //    /* 11:  {  */ ( Initial,    YieldBraceL    ),
    //    /* 12:  }  */ ( Initial,    YieldBraceR    ),
    //    /* 13:  (  */ ( Initial,    YieldParenL    ),
    //    /* 14:  )  */ ( Initial,    YieldParenR    ),
    //    /* 15:  [  */ ( Initial,    YieldBracketL  ),
    //    /* 16:  ]  */ ( Initial,    YieldBracketR  ),
    //    /* 17:  .  */ ( AfterDot,   Skip           ),
    //    /* 18:  @  */ ( Initial,    YieldAt        ),
    //    /* 19:  !  */ ( AfterBang,  Skip           ),
    //    /* 20:  ~  */ ( Initial,    YieldTilde     ),
    //    /* 21:  ?  */ ( Initial,    YieldQuestion  ),
    //    /* 22:  *  */ ( Initial,    YieldStar      ),
    //    /* 23:  /  */ ( Initial,    YieldSlash     ),
    //    /* 24:  %  */ ( Initial,    YieldPercent   ),
    //    /* 25:  +  */ ( AfterPlus,  Skip           ),
    //    /* 26:  -  */ ( AfterMinus, Skip           ),
    //    /* 27:  &  */ ( Initial,    YieldAmpersand ),
    //    /* 28:  ^  */ ( Initial,    YieldCaret     ),
    //    /* 29:  |  */ ( Initial,    YieldPipe      ),
    //    /* 30:  <  */ ( AfterLess,  Skip           ),
    //    /* 31:  >  */ ( AfterMore,  Skip           ),
    //    /* 32:  =  */ ( AfterEqual, Skip           ),
    //    /* 33:  :  */ ( Initial,    YieldColon     ),
    //    /* 34:  ,  */ ( Initial,    YieldComma     ),
    //]),

    // InSpace - in whitespace
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

    // AfterEos - after end of statement
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

    // InIdOrKw - in identifier or keyword
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
        3, 2, 2, 2, 2, 2, 2, 2,  2, 2, 8, 8, 8, 8, 8, 8, // 01234567 89:;<=>?
        8, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
        x, x, x, x, x, x, x, x,  x, x, x, 8, 8, 8, 8, 3, // PQRSTUVW XYZ[\]^_
        8, x, 6, x, x, x, x, x,  x, x, x, x, x, x, x, 5, // `abcdefg hijklmno
        x, x, x, x, x, x, x, x,  4, x, x, 8, 8, 8, 8, x, // pqrstuvw xyz{|}~. <- DEL
    ],&[
        //             State     Action
        /* 0: eof */ ( AtEof,    YieldNum        ),
        /* 1: ??? */ ( AtEof,    ErrorInvalidNum ),
        /* 2: 1-9 */ ( InNumDec, AccumNumDec     ),
        /* 3: 0_  */ ( InNumDec, Skip            ),
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
        /* 4: \s  */ ( InSpace,  YieldNum        ),
        /* 5: opr */ ( Initial,  YieldNum        ),
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
        /* 4: \s  */ ( InSpace,  YieldNum        ),
        /* 5: opr */ ( Initial,  YieldNum        ),
    ]),

    // InChar - in a character literal
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

    // AtCharEnd - expecting ' to end a character literal
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

    // InStr - in a string literal
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

    // InEsc - after \ in a character or string literal
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
        /*  2:  0  */ ( InStr,     AccumEscNul     ), // pops state
        /*  3:  n  */ ( InStr,     AccumEscLf      ), // pops state
        /*  4:  r  */ ( InStr,     AccumEscCr      ), // pops state
        /*  5:  t  */ ( InStr,     AccumEscTab     ), // pops state
        /*  6:  \  */ ( InStr,     AccumEscChar    ), // pops state
        /*  7:  '  */ ( InStr,     AccumEscChar    ), // pops state
        /*  8:  "  */ ( InStr,     AccumEscChar    ), // pops state
        /*  9:  x  */ ( AtEscHex0, Skip            ),
        /* 10:  u  */ ( AtEscUni0, Skip            ),
    ]),

    // AtEscHex0 - after \x in a character or string literal
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

    // AtEscHex1 - after \x and a hex digit in a character or string literal
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
        /* 2: 0-9 */ ( InStr, AccumEscHexDig  ), // pops state
        /* 3: A-F */ ( InStr, AccumEscHexUc   ), // pops state
        /* 4: a-f */ ( InStr, AccumEscHexLc   ), // pops state
    ]),

    // AtEscUni0 - after \u in a character or string literal, expecting {
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

    // AtEscUni1 - after \u{ in a character or string literal
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
        /* 5:  }  */ ( InStr,     AccumEscNum     ), // pops state
    ]),

    //// AfterDot <( . )> : after a dot
    //([
    //    x, x, x, x, x, x, x, x,  x, 2, x, x, x, 2, x, x, // ........ .tn..r..
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
    //    2, 4, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, 5, x, 6, // 01234567 89:;<=>?
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, 3, x, // pqrstuvw xyz{|}~. <- DEL
    //],&[
    //    //             State    Action
    //    /* 0: eof */ ( AtEof,   YieldDot         ),
    //    /* 1: ??? */ ( Initial, YieldDot         ),
    //    /* 2: \s  */ ( Initial, YieldDot         ),
    //    /* 3:  ~  */ ( Initial, YieldDotTilde    ),
    //    /* 4:  !  */ ( Initial, YieldDotBang     ),
    //    /* 5:  =  */ ( Initial, YieldDotEqual    ),
    //    /* 6:  ?  */ ( Initial, YieldDotQuestion ),
    //]),

    //// AfterPlus <( + )> : after a plus sign
    //([
    //    x, x, x, x, x, x, x, x,  x, 2, x, x, x, 2, x, x, // ........ .tn..r..
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
    //    2, x, x, x, x, x, x, x,  x, x, x, 3, x, x, x, x, //  !"#$%&' ()*+,-./
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // 01234567 89:;<=>?
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    //],&[
    //    //             State    Action
    //    /* 0: eof */ ( AtEof,   YieldPlus     ),
    //    /* 1: ??? */ ( Initial, YieldPlus     ),
    //    /* 2: \s  */ ( Initial, YieldPlus     ),
    //    /* 3:  +  */ ( Initial, YieldPlusPlus ),
    //]),

    //// AfterMinus <( - )> : after a minus sign
    //([
    //    x, x, x, x, x, x, x, x,  x, 2, x, x, x, 2, x, x, // ........ .tn..r..
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
    //    2, x, x, x, x, x, x, x,  x, x, x, x, x, 3, x, x, //  !"#$%&' ()*+,-./
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, 4, x, // 01234567 89:;<=>?
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    //],&[
    //    //             State    Action
    //    /* 0: eof */ ( AtEof,   YieldMinus      ),
    //    /* 1: ??? */ ( Initial, YieldMinus      ),
    //    /* 2: \s  */ ( Initial, YieldMinus      ),
    //    /* 3:  -  */ ( Initial, YieldMinusMinus ),
    //    /* 4:  >  */ ( Initial, YieldMinusArrow ),
    //]),

    //// AfterLess <( < )> : after a less-than sign
    //([
    //    x, x, x, x, x, x, x, x,  x, 2, x, x, x, 2, x, x, // ........ .tn..r..
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
    //    2, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
    //    x, x, x, x, x, x, x, x,  x, x, x, x, 3, 5, 4, x, // 01234567 89:;<=>?
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    //],&[
    //    //             State    Action
    //    /* 0: eof */ ( AtEof,   YieldLess      ),
    //    /* 1: ??? */ ( Initial, YieldLess      ),
    //    /* 2: \s  */ ( Initial, YieldLess      ),
    //    /* 3:  <  */ ( Initial, YieldLessLess  ),
    //    /* 4:  >  */ ( Initial, YieldLessMore  ),
    //    /* 5:  =  */ ( Initial, YieldLessEqual ),
    //]),

    //// AfterMore <( > )> : after a greater-than sign
    //([
    //    x, x, x, x, x, x, x, x,  x, 2, x, x, x, 2, x, x, // ........ .tn..r..
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
    //    2, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
    //    x, x, x, x, x, x, x, x,  x, x, x, x, 3, 5, 4, x, // 01234567 89:;<=>?
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    //],&[
    //    //             State    Action
    //    /* 0: eof */ ( AtEof,   YieldMore      ),
    //    /* 1: ??? */ ( Initial, YieldMore      ),
    //    /* 2: \s  */ ( Initial, YieldMore      ),
    //    /* 3:  <  */ ( Initial, YieldMoreMore  ), // TODO: exchange
    //    /* 4:  >  */ ( Initial, YieldMoreMore  ),
    //    /* 5:  =  */ ( Initial, YieldMoreEqual ),
    //]),

    //// AfterEqual <( = )> : after an equal sign
    //([
    //    x, x, x, x, x, x, x, x,  x, 2, x, x, x, 2, x, x, // ........ .tn..r..
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
    //    2, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, 4, 3, x, // 01234567 89:;<=>?
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    //],&[
    //    //             State    Action
    //    /* 0: eof */ ( AtEof,   YieldEqual      ),
    //    /* 1: ??? */ ( Initial, YieldEqual      ),
    //    /* 2: \s  */ ( Initial, YieldEqual      ),
    //    /* 3:  >  */ ( Initial, YieldEqualArrow ),
    //    /* 4:  =  */ ( Initial, YieldEqualEqual ),
    //]),

    //// AfterBang <( ! )> : after a bang mark
    //([
    //    x, x, x, x, x, x, x, x,  x, 2, x, x, x, 2, x, x, // ........ .tn..r..
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // ........ ........
    //    2, 4, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, //  !"#$%&' ()*+,-./
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, 3, x, x, // 01234567 89:;<=>?
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // @ABCDEFG HIJKLMNO
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // PQRSTUVW XYZ[\]^_
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // `abcdefg hijklmno
    //    x, x, x, x, x, x, x, x,  x, x, x, x, x, x, x, x, // pqrstuvw xyz{|}~. <- DEL
    //],&[
    //    //             State      Action
    //    /* 0: eof */ ( AtEof,     YieldBang      ),
    //    /* 1: ??? */ ( Initial,   YieldBang      ),
    //    /* 2: \s  */ ( Initial,   YieldBang      ),
    //    /* 3:  =  */ ( Initial,   YieldBangEqual ),
    //    /* 4:  !  */ ( AfterBang, YieldBang2     ), // need to consume!
    //]),

//  // State - Description
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
// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use aex::token::Token::*;

    #[test]
    fn empty() {
        lex("", |it| { it.yields(Eof); });
    }

    #[test]
    fn space() {
        lex( " \r\t" , |it| { it                            .yields(Eof); });
        lex( " \r\t1", |it| { it.yields_int(1)              .yields(Eof); });
        lex("1 \r\t" , |it| { it.yields_int(1)              .yields(Eof); });
        lex("1 \r\t2", |it| { it.yields_int(1).yields_int(2).yields(Eof); });
    }

    #[test]
    fn eos() {
        lex(";"         , |it| { it.yields(Eos)              .yields(Eof); });
        lex("\n"        , |it| { it.yields(Eos)              .yields(Eof); });
        lex(";1"        , |it| { it.yields(Eos).yields_int(1).yields(Eof); });
        lex("\n1"       , |it| { it.yields(Eos).yields_int(1).yields(Eof); });
        lex("; \r\t\n;" , |it| { it.yields(Eos)              .yields(Eof); });
        lex("\n \r\t\n;", |it| { it.yields(Eos)              .yields(Eof); });
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
        lex( "123456789", |it| { it.yields_int( 123456789).yields(Eof); });
        lex("0x01234567", |it| { it.yields_int(0x01234567).yields(Eof); });
        lex("0x89ABCDEF", |it| { it.yields_int(0x89ABCDEF).yields(Eof); });
        lex("0x89abcdef", |it| { it.yields_int(0x89ABCDEF).yields(Eof); });
        lex("0o01234567", |it| { it.yields_int(0o01234567).yields(Eof); });
        lex(      "0b01", |it| { it.yields_int(      0b01).yields(Eof); });
        lex(       "012", |it| { it.yields_int(        12).yields(Eof); });
        lex(         "0", |it| { it.yields_int(         0).yields(Eof); });
        lex(    "1__2__", |it| { it.yields_int(        12).yields(Eof); });
        lex("0x__1__2__", |it| { it.yields_int(      0x12).yields(Eof); });
        lex(       "0__", |it| { it.yields_int(         0).yields(Eof); });
        lex(     "0b1  ", |it| { it.yields_int(         1).yields(Eof); });
        lex(      "0b1;", |it| { it.yields_int(         1).yields(Eos).yields(Eof); });
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
    fn punctuation() {
        lex("{}", |it| { it.yields(BraceL)  .yields(BraceR);   });
        lex("()", |it| { it.yields(ParenL)  .yields(ParenR);   });
        lex("[]", |it| { it.yields(BracketL).yields(BracketR); });
//        lex("{ } ( ) [ ] . @ ++ -- ! ~ ? * / % + - << >> & ^ | .~ .! .= .? \
//             <> == != < > <= >= => -> = : ,", |it| { it
//            .yields(BraceL)     .yields(BraceR)     .yields(ParenL)    .yields(ParenR)
//            .yields(BracketL)   .yields(BracketR)   .yields(Dot)       .yields(At)
//            .yields(PlusPlus)   .yields(MinusMinus)
//            .yields(Bang)       .yields(Tilde)      .yields(Question)
//            .yields(Star)       .yields(Slash)      .yields(Percent)
//            .yields(Plus)       .yields(Minus)
//            .yields(LessLess)   .yields(MoreMore)
//            .yields(Ampersand)  .yields(Caret)      .yields(Pipe)
//            .yields(DotTilde)   .yields(DotBang)    .yields(DotEqual)  .yields(DotQuestion)
//            .yields(LessMore)   .yields(EqualEqual) .yields(BangEqual)
//            .yields(Less)       .yields(More)       .yields(LessEqual) .yields(MoreEqual)
//            .yields(EqualArrow) .yields(MinusArrow) .yields(Equal)
//            .yields(Colon)      .yields(Comma)
//            .yields(Eof);
//        });
    }

    // Test Harness

    use num::BigInt;
    //use aex::compiler::Compiler;
    use aex::source::File;
    use aex::token::*;

    fn lex<'a, F>(input: &'a str, assert: F)
    where F: FnOnce(&mut LexerHarness) {
        let     file     = File::new("test", input);
        let     compiler = Compiler::new();
        let     lexer    = Lexer::new(&compiler, &file);
        let mut harness  = LexerHarness(lexer);

        assert(&mut harness)
    }

    struct LexerHarness<'a> (Lexer<'a>);

    impl<'a> LexerHarness<'a> {
        fn yields(&mut self, token: Token) -> &mut Self {
            assert_eq!(token, self.0.lex().1);
            self
        }

        fn yields_int(&mut self, n: u64) -> &mut Self {
            self.yields(Int(BigInt::from(n)))
        }

        fn yields_error(&mut self) -> &mut Self {
            self.yields(Error)
        }
    }
}

