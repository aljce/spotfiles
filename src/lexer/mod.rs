pub mod token;
pub mod iterator;

use lexer::token::{Token};
use lexer::iterator::{TokenIterator};

pub trait Lex: AsRef<str> {
    fn lex(&self) -> Vec<Token> {
        TokenIterator::new(self.as_ref()).collect()
    }
}

impl<T: AsRef<str>> Lex for T {}
