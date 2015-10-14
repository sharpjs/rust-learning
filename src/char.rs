#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum CharClass {
    Eof   , // [ 0] end-of-file virtual character
    Other , // [ 1] characters not in any class below

    Space , // [ 2] space: general
    NL    , // [ 3] space: newline

    LCurl , // [ 4] punctuation: {
    RCurl , // [ 5] punctuation: }
    LPar  , // [ 6] punctuation: (
    RPar  , // [ 7] punctuation: )
    LBrak , // [ 8] punctuation: [
    RBrak , // [ 9] punctuation: ]
    DQuot , // [10] punctuation: "
    SQuot , // [11] punctuation: '
    BkSla , // [12] punctuation: \
    Dot   , // [13] punctuation: .
    At    , // [14] punctuation: @
    Bang  , // [15] punctuation: !
    Tilde , // [16] punctuation: ~
    BkTik , // [17] punctuation: `
    Star  , // [18] punctuation: *
    Slash , // [19] punctuation: /
    Pct   , // [20] punctuation: %
    Plus  , // [21] punctuation: +
    Minus , // [22] punctuation: -
    Amper , // [23] punctuation: &
    Caret , // [24] punctuation: ^
    Pipe  , // [25] punctuation: |
    LT    , // [26] punctuation: <
    GT    , // [27] punctuation: >
    Equal , // [28] punctuation: =
    Dollr , // [29] punctuation: $
    Quest , // [30] punctuation: ?
    Colon , // [31] punctuation: :
    Comma , // [32] punctuation: ,
    Semi  , // [33] punctuation: ;

    Under , // [34] letter: _
    LoB   , // [35] letter: b
    LoD   , // [36] letter: d
    LoO   , // [37] letter: o
    LoX   , // [38] letter: x
    Hex   , // [39] letter: a_cdef ABCDEF
    Alpha , // [40] letter: other

    Zero  , // [41] digit: 0
    One   , // [42] digit:  1
    Oct   , // [43] digit:   234567
    Dec   , // [44] digit:         89
}
use self::CharClass::*;

pub const CHAR_CLASS_COUNT: usize = 45;

// Character classes for 7-bit ASCII
//
static CHAR_CLASSES: [CharClass; 128] = [
    Other, Other, Other, Other, Other, Other, Other, Other, // ........
    Other, Space, NL   , Other, Other, Space, Other, Other, // .tn..r..
    Other, Other, Other, Other, Other, Other, Other, Other, // ........
    Other, Other, Other, Other, Other, Other, Other, Other, // ........
    Space, Bang , DQuot, Other, Dollr, Pct  , Amper, SQuot, //  !"#$%&'
    LPar , RPar , Star , Plus , Comma, Minus, Dot  , Slash, // ()*+,-./
    Zero , One  , Oct  , Oct  , Oct  , Oct  , Oct  , Oct  , // 01234567
    Dec  , Dec  , Semi , Colon, LT   , Equal, GT   , Quest, // 89:;<=>?
    At   , Hex  , LoB  , Hex  , LoD  , Hex  , Hex  , Alpha, // @ABCDEFG
    Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, LoO  , // HIJKLMNO
    Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, // PQRSTUVW
    LoX  , Alpha, Alpha, LBrak, BkSla, RBrak, Caret, Under, // XYZ[\]^_
    BkTik, Hex  , Hex  , Hex  , Hex  , Hex  , Hex  , Alpha, // `abcdefg
    Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, // hijklmno
    Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, // pqrstuvw
    Alpha, Alpha, Alpha, LCurl, Pipe , RCurl, Tilde, Other, // xyz{|}~. <- DEL
];

pub trait Classify {
    fn classify(self) -> (CharClass, char);
}

impl Classify for char {
    #[inline]
    fn classify(self) -> (CharClass, char) {
        let index = self as usize;
        let class = if (index & 0x7F) == index {
            // 7-bit ASCII chars are classified individually
            CHAR_CLASSES[index]
        } else {
            // All other chars are considered "Other"
            Other
        };
        (class, self)
    }
}

impl Classify for Option<char> {
    #[inline]
    fn classify(self) -> (CharClass, char) {
        match self {
            Some(c) => c.classify(),
            None    => (Eof, '\0')
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::CharClass::*;

    #[test]
    fn classify_space() {
        assert_eq!(' '.classify(), (Space, ' '));
    }

    #[test]
    fn classify_lf() {
        assert_eq!('\n'.classify(), (NL, '\n'));
    }

    #[test]
    fn classify_cr() {
        assert_eq!('\r'.classify(), (Space, '\r'));
    }

    #[test]
    fn classify_digit() {
        assert_eq!('9'.classify(), (Dec, '9'));
    }

    #[test]
    fn classify_some() {
        assert_eq!(Some('.').classify(), (Dot, '.'));
    }

    #[test]
    fn classify_none() {
        assert_eq!(None.classify(), (Eof, '\0'));
    }
}

