pub mod token;
pub mod iterator;

use lexer::iterator::{TokenIterator};

pub trait Lex: AsRef<str> {
    fn lex(&self) -> TokenIterator {
        TokenIterator::new(self.as_ref())
    }
}

impl<T: AsRef<str>> Lex for T {}
