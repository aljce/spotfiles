use std::str::CharIndices;
use itertools::Itertools;
use lexer::token::{Location, MoveKind, Token, NamePart};
use self::State::*;
use std::cmp::Ordering::*;

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
    fn take_str(&self, slice: &Slice) -> &'a str {
        &self.chars[slice.start .. slice.end + 1]
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

struct Slice {
    start: usize,
    end:   usize
}

enum State {
    Newline,
    Normal,
    PartialName {
        start_loc: Location,
        slices: Vec<NamePart<Slice>>
    }
}

pub struct TokenIterator<'a> {
    chars_loc:   CharsLoc<'a>,
    state:       State,
    indentation: usize,
    token_stack: Vec<Token<'a>>
}

impl<'a> TokenIterator<'a> {
    pub fn new<'b>(input: &'b str) -> TokenIterator<'b> {
        TokenIterator {
            chars_loc:   CharsLoc::new(input),
            state:       Newline,
            indentation: 0,
            token_stack: Vec::new()
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
                    .take_while_ref(|&(_,c)| c == '\t' || c == ' ')
                    .count() + 1;
                self.state = Normal;
                let indentation = self.indentation;
                match indentation.cmp(&depth) {
                    Less => {
                        self.indentation = depth;
                        Some(Token::indent(self.chars_loc.location, depth - indentation))
                    },
                    Equal => self.next(),
                    Greater => {
                        self.indentation = depth;
                        Some(Token::de_indent(self.chars_loc.location, indentation - depth))
                    }
                }
            },
            PartialName { ref slices, ref start_loc } => {
                use lexer::token::NamePart::*;
                let with_strs = (*slices).iter().map(
                    | part | {
                        match *part {
                            Ident(ref slice) => Ident(self.chars_loc.take_str(slice)),
                            Star => Star,
                            Ampersand => Ampersand,
                            Slash => Slash
                        }
                    }
                ).collect();
                Some(Token::name(*start_loc, with_strs))
            },
            Normal => self.next()
        }
    }

    fn move_kind(&mut self, start: usize, move_kind: MoveKind) -> Option<Token<'a>> {
        self.state = Normal;
        match move_kind {
            MoveKind::Copy => match self.chars_loc.next() {
                Some((_,c)) if c == ' ' || c == '\t' =>
                    Some(Token::move_kind(self.chars_loc.location, move_kind)),
                _ => self.name(start)

            },
            MoveKind::Link => match self.chars_loc.next() {
                Some((_,c)) if c == '>' =>
                    Some(Token::move_kind(self.chars_loc.location, move_kind)),
                _ => self.name(start)
            }
        }
    }

    fn set_partial(&mut self, name_part: NamePart<Slice>) -> Option<Token<'a>> {
        let mut slices = Vec::new();
        slices.push(name_part);
        self.state = PartialName {
            start_loc: self.chars_loc.location,
            slices:    slices
        };
        let indentation = self.indentation;
        match self.state {
            Newline if indentation != 0 => {
                self.indentation = 0;
                Some(Token::de_indent(self.chars_loc.location, indentation))
            },
            _ => self.next()
        }
    }

    fn single_name(&mut self, name_part: NamePart<Slice>) -> Option<Token<'a>> {
        match self.state {
            PartialName { start_loc, ref mut slices } => {
                slices.push(name_part);
            },
            _ => return self.set_partial(name_part)
        }
        self.next()
    }

    fn name(&mut self, index: usize) -> Option<Token<'a>> {
        use lexer::token::NamePart::*;
        match self.state {
            PartialName { start_loc, ref mut slices } => {
                // match slices.last_mut() {
                //     Some(name_part) => {
                //         if let Ident(ref mut slice) = *name_part {
                //             slice.end = index;
                //         }
                //     },
                //     _ => {
                //         slices.push(Ident(Slice {
                //             start: index,
                //             end:   index
                //         }))
                //     }
                // }
                let was_none = match slices.last_mut() {
                    Some(name_part) => {
                        if let Ident(ref mut slice) = *name_part {
                            slice.end = index;
                        };
                        false
                    },
                    _ => true
                };
                if was_none {
                    slices.push(Ident(Slice {
                        start: index,
                        end:   index
                    }));
                }
            },
            _ => return self.set_partial(Ident(Slice {
                start: index,
                end:   index
            }))
        }
        self.next()
    }

    fn finalizer(&mut self) -> Option<Token<'a>> {
        let indentation = self.indentation;
        if indentation > 0 {
            self.indentation = 0;
            Some(Token::de_indent(self.chars_loc.location, indentation))
        } else {
            None
        }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.token_stack.pop().or(match self.chars_loc.next() {
            Some((index,c)) => match c {
                '#'  => Some(self.line_comment()),
                '\n' => Some(self.newline()),
                ' ' | '\t' => self.white_space(),
                '>'  => self.move_kind(index,MoveKind::Copy),
                '-'  => self.move_kind(index,MoveKind::Link),
                '*'  => self.single_name(NamePart::Star),
                '&'  => self.single_name(NamePart::Ampersand),
                '\\' => self.single_name(NamePart::Slash),
                _    => self.name(index)
            },
            None => self.finalizer()
        })
    }
}
