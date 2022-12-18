use std::str::Chars;

#[derive(Debug)]
pub struct StringReader<'a> {
  src: Chars<'a>,
  curr: Option<char>,
  line: usize,
  pos: usize,
}

impl<'a> StringReader<'a> {
  pub fn new(src: &'a str) -> Self {
    let mut src = src.chars();
    let curr = src.next();
    Self {
      src,
      curr,
      line: 1,
      pos: 1,
    }
  }

  pub fn peek(&self) -> Option<char> {
    self.curr
  }

  pub fn next(&mut self) -> Option<char> {
    let res = self.curr;
    self.advance();
    res
  }

  pub fn advance(&mut self) {
    if let Some(sym) = self.curr {
      if sym == '\n' {
        self.line += 1;
        self.pos = 1;
      } else {
        self.pos += 1;
      }
      self.curr = self.src.next();
    }
  }

  pub fn position(&self) -> (usize, usize) {
    (self.line, self.pos)
  }
}

#[cfg(test)]
pub mod tests {
  use super::*;

  #[test]
  fn test_reader() {
    let mut s = StringReader::new("hello");
    assert!(s.peek() == Some('h'));
    assert!(s.peek() == Some('h'));
    assert!(s.next() == Some('h'));
    assert!(s.peek() == Some('e'));
    assert!(s.next() == Some('e'));
    assert!(s.next() == Some('l'));
    assert!(s.next() == Some('l'));
    assert!(s.next() == Some('o'));
    assert!(s.peek() == None);
    assert!(s.next() == None);
    assert!(s.next() == None);
  }
}
