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
    C     , // [35] letter: c
    E     , // [36] letter: e
    F     , // [37] letter: f
    H     , // [38] letter: h
    I     , // [39] letter: i
    J     , // [40] letter: j
    L     , // [41] letter: l
    M     , // [42] letter: m
    N     , // [43] letter: n
    O     , // [44] letter: o
    P     , // [45] letter: p
    R     , // [46] letter: r
    S     , // [47] letter: s
    T     , // [48] letter: t
    U     , // [49] letter: u
    W     , // [50] letter: w
    Y     , // [51] letter: y
    Alpha , // [52] letter: other (abdgkqvxz or uppercase)

    Zero  , // [53] digit: 0
    One   , // [54] digit: 1
    Digit , // [55] digit: other (23456789)
}
use self::CharClass::*;

// Character classes for 7-bit ASCII
//
static CHAR_CLASSES: [CharClass; 128] = [
    Other, Other, Other, Other, Other, Other, Other, Other, // ........
    Other, Space, NL   , Other, Other, Space, Other, Other, // .tn..r..
    Other, Other, Other, Other, Other, Other, Other, Other, // ........
    Other, Other, Other, Other, Other, Other, Other, Other, // ........
    Space, Bang , DQuot, Other, Dollr, Pct  , Amper, SQuot, //  !"#$%&'
    LPar , RPar , Star , Plus , Comma, Minus, Dot  , Slash, // ()*+,-./
    Zero , One  , Digit, Digit, Digit, Digit, Digit, Digit, // 01234567
    Digit, Digit, Semi , Colon, LT   , Equal, GT   , Quest, // 89:;<=>?
    At   , Alpha, Alpha, C    , Alpha, E    , F    , Alpha, // @ABCDEFG
    H    , I    , J    , Alpha, L    , M    , N    , O    , // HIJKLMNO
    P    , Alpha, R    , S    , T    , U    , Alpha, W    , // PQRSTUVW
    Alpha, Y    , Alpha, LBrak, BkSla, RBrak, Caret, Under, // XYZ[\]^_
    BkTik, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, // `abcdefg
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
        assert_eq!('9'.classify(), (Digit, '9'));
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

