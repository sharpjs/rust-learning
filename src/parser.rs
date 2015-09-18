
use token::Token;
use token::Token::*;

const VALUE_INIT_CAPACITY: usize = 128; // bytes

pub struct Parser<I> where I: Iterator<Item=char> {
  chars: I,
  c:     char,
  start: usize,
  index: usize,
  value: String
}

trait Parseable {
  fn parser(&self) -> Parser<Self>;
}

impl<I> Parseable for I where I: Iterator<Item=char> + Clone {
  fn parser(&self) -> Parser<I> {
    let mut cs = self.clone();
    let     c  = cs.next().unwrap(); // Should not use unwrap
    Parser {
      chars: cs,
      c:     c,
      start: 0,
      index: 0,
      value: String::with_capacity(VALUE_INIT_CAPACITY)
    }
  }
}

impl<'a, I> Parser<I> where I: 'a + Iterator<Item=char> {

  fn parse(&mut self) -> Token<'a> {
    self.parse_id()
  }

  fn parse_expr(&mut self) /*-> Token*/ {
    let l = self.parse_id();
    let r = self.parse_id();
  }

  fn parse_id(&mut self) -> Token<'a> {
    self.value.clear();
    loop {
      match self.c {
        c@'a'...'z' | c@'A'...'Z' | c@'0'...'9' | c@'_' => {
          self.value.push(c);
          self.consume();
        },
        _ => return Id(self.text())
      }
    }
  }

  fn text(&mut self) -> String {
    self.value.clone()
  }

  fn consume(&mut self) {
    self.index += self.c.len_utf8();
    self.c      = match self.chars.next() {
      Some(c) => c,
      None    => '\0'
    }
  }
}

#[test]
fn test_1() {
  assert_eq!("abc  ".chars().parser().parse_id(), Id("abc".into()));
}

