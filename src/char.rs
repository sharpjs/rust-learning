#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum CharClass {
    Eof,    // end-of-file virtual character
    Other,  // characters not in any class below
    Space,  // whitespace: general
    LF,     // whitespace: line feed
    CR,     // whitespace: carriage return
    Alpha,  // letter
    Digit,  // digit
    Plus,   // +
    Minus,  // -
}
use self::CharClass::*;

// Character classes for 7-bit ASCII
//
static CHAR_CLASSES: [CharClass; 128] = [
    Other, Other, Other, Other, Other, Other, Other, Other, // ........
    Other, Space, LF   , Other, Other, CR   , Other, Other, // .tn..r..
    Other, Other, Other, Other, Other, Other, Other, Other, // ........
    Other, Other, Other, Other, Other, Other, Other, Other, // ........
    Space, Other, Other, Other, Other, Other, Other, Other, //  !"#$%&'
    Other, Other, Other, Plus , Other, Minus, Other, Other, // ()*+,-./
    Digit, Digit, Digit, Digit, Digit, Digit, Digit, Digit, // 01234567
    Digit, Digit, Other, Other, Other, Other, Other, Other, // 89:;<=>?
    Other, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, // @ABCDEFG
    Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, // HIJKLMNO
    Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, // PQRSTUVW
    Alpha, Alpha, Alpha, Other, Other, Other, Other, Other, // XYZ[\]^_
    Other, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, // `abcdefg
    Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, // hijklmno
    Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, Alpha, // pqrstuvw
    Alpha, Alpha, Alpha, Other, Other, Other, Other, Other, // xyz{|}~. <- DEL
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
        assert_eq!('\n'.classify(), (LF, '\n'));
    }

    #[test]
    fn classify_cr() {
        assert_eq!('\r'.classify(), (CR, '\r'));
    }

    #[test]
    fn classify_digit() {
        assert_eq!('9'.classify(), (Digit, '9'));
    }
}

