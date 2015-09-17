
use token::Token;
use token::Token::*;

pub struct Parser<I> where I: Iterator<Item=u8> {
  bytes: I,
  byte:  u8,
  start: usize,
  index: usize
}

trait Parseable {
  fn parser(&self) -> Parser<Self>;
}

impl<'a, I> Parseable for I where I: 'a + Iterator<Item=u8> + Clone {
  fn parser(&self) -> Parser<I> {
    let mut bytes = self.clone();
    let     byte  = bytes.next().unwrap();
    Parser { bytes: bytes, byte: byte, start: 0, index: 0 }
  }
}

impl<'a, I> Parser<I> where I: 'a + Iterator<Item=u8> {

  fn parse(&mut self) -> Token<'a> {
    self.parse_id()
  }

  fn parse_expr(&mut self) /*-> Token*/ {
    let l = self.parse_id();
    let r = self.parse_id();
  }

  fn parse_id(&mut self) -> Token<'a> {
    loop {
      match self.byte {
        b'a'...b'z' | b'A'...b'Z' | b'0'...b'9' | b'_' => self.consume(),
        _ => return Id("hi" /*self.bytes[self.start ... self.index]*/ )
      }
    }
  }

  fn consume(&mut self) {
    self.byte   = self.bytes.next().unwrap();
    self.index += 1; //self.char.len_utf8();
  }
}

#[test]
fn test_1() {
  assert_eq!("abc  ".bytes().parser().parse_id(), Id("hi"));
}

