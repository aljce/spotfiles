use std::str::CharIndices;
use lexer::token::{Location, MoveKind, Token};
use self::State::*;
use itertools::Itertools;

#[derive(Clone)]
struct CharsLoc<'a> {
    char_stream: CharIndices<'a>,
    chars:       &'a str,
    location:    Location,
}

impl<'a> CharsLoc<'a> {
    fn new<'b>(input: &'b str) -> CharsLoc<'b> {
        CharsLoc {
            char_stream: input.char_indices(),
            chars:       input,
            location:    Location { line:   0,
                                    column: 0 },
        }
    }
    fn take_str(&self, start: usize, end: usize) -> &'a str {
        &self.chars[start .. end + 1]
    }
}

impl<'a> Iterator for CharsLoc<'a> {
    type Item = (usize, char);
    fn next(&mut self) -> Option<Self::Item> {
        let may_char = self.char_stream.next();
        match may_char {
            Some((_,'\n')) => {
                self.location.line += 1;
                self.location.column = 0;
            },
            Some(_) => {
                self.location.column += 1;
            },
            None => ()
        }
        may_char
    }
}

enum State {
    Newline,
    Normal
}

pub struct TokenIterator<'a> {
    chars_loc:   CharsLoc<'a>,
    state:       State,
    indentation: usize,
    token_stack: Option<Token<'a>>
}

fn is_reserved(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\n' || c == '#'
}

impl<'a> TokenIterator<'a> {
    pub fn new<'b>(input: &'b str) -> TokenIterator<'b> {
        TokenIterator {
            chars_loc:   CharsLoc::new(input),
            state:       Newline,
            indentation: 0,
            token_stack: None
        }
    }

    fn line_comment(&mut self) -> Token<'a> {
        let comment_loc = self.chars_loc.location;
        (&mut self.chars_loc)
            .take_while(|&(_,c)| c != '\n')
            .last();
        self.state = Newline;
        Token::newline(comment_loc)
    }

    fn newline(&mut self) -> Token<'a> {
        self.state = Newline;
        Token::newline(self.chars_loc.location)
    }

    fn white_space(&mut self) -> Option<Token<'a>> {
        match self.state {
            Newline => {
                let depth = (&mut self.chars_loc)
                    .take_while_ref(|&(_,c)| c == '\t' && c == ' ')
                    .count();
                self.state = Normal;
                let indentation = self.indentation;
                if indentation == depth {
                    self.next()
                } else {
                    self.indentation = depth;
                    Some(Token::indent(self.chars_loc.location, depth - indentation))
                }
            },
            Normal => self.next()
        }
    }

    fn name(&mut self, start: usize) -> Token<'a> {
        let location = self.chars_loc.location;
        let end = (&mut self.chars_loc)
            .take_while_ref(|&(_,c)| !is_reserved(c))
            .last()
            .map(|(end,_)| end);
        let token = Token::name(location,
                                self.chars_loc.take_str(start, end.unwrap_or(start)));
        match self.state {
            Newline => {
                self.state = Normal;
                let indentation = self.indentation;
                self.indentation = 0;
                self.token_stack = Some(token);
                Token::de_indent(self.chars_loc.location, indentation)
            },
            Normal => token
        }
    }

    fn move_kind(&mut self, start: usize, move_kind: MoveKind) -> Token<'a> {
        self.state = Normal;
        match move_kind {
            MoveKind::Copy => match self.chars_loc.next() {
                Some((_,c)) if c == ' ' || c == '\t' =>
                    Token::move_kind(self.chars_loc.location, move_kind),
                _ => self.name(start)

            },
            MoveKind::Link => match self.chars_loc.next() {
                Some((_,c)) if c == '>' =>
                    Token::move_kind(self.chars_loc.location, move_kind),
                _ => self.name(start)
            }
        }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.token_stack.take().or(match self.chars_loc.next() {
            Some((index,c)) => match c {
                '#'  => Some(self.line_comment()),
                '\n' => Some(self.newline()),
                ' ' | '\t' => self.white_space(),
                '>'  => Some(self.move_kind(index,MoveKind::Copy)),
                '-'  => Some(self.move_kind(index,MoveKind::Link)),
                _    => Some(self.name(index))
            },
            None => None
        })
    }
}
