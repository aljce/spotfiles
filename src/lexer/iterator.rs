use std::str::CharIndices;
use itertools::Itertools;
use lexer::token::{Location, MoveKind, Token, NamePart};
use self::State::*;
use std::cmp::Ordering::*;

fn log_on() -> bool { false }

macro_rules! debug {
    ( $msg:expr ) => {
        if log_on() {
            println!("{}",$msg);
        }
    };
    ( $msg:expr,$state:expr ) => {
        if log_on() {
            println!("{}, (state: {:?})", $msg, $state);
        }
    };
}

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

#[derive(Debug,PartialEq)]
struct Slice {
    start: usize,
    end:   usize
}

#[derive(Debug,PartialEq)]
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
    token_stack: Option<Token<'a>>
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
        debug!("line_comment",self.state);
        let comment_loc = self.chars_loc.location;
        (&mut self.chars_loc)
            .take_while(|&(_,c)| c != '\n')
            .last();
        self.state = Newline;
        Token::newline(comment_loc)
    }

    fn to_str_slices(&self, start_loc: &Location, slices: &Vec<NamePart<Slice>>) -> Token<'a> {
        use lexer::token::NamePart::*;
        let with_strs = slices.iter().map(
            | part | {
                match *part {
                    Ident(ref slice) => Ident(self.chars_loc.take_str(slice)),
                    Star => Star,
                    Ampersand => Ampersand,
                    Slash => Slash
                }
            }
        ).collect();
        Token::name(*start_loc, with_strs)
    }

    fn newline(&mut self) -> Token<'a> {
        debug!("newline",self.state);
        let newline_token = Token::newline(self.chars_loc.location);
        let token = match self.state {
            PartialName { ref start_loc, ref slices } => {
                self.token_stack = Some(newline_token);
                self.to_str_slices(start_loc, slices)
            },
            _ => newline_token
        };
        self.state = Newline;
        token
    }

    fn white_space(&mut self, delta: usize) -> Option<Token<'a>> {
        debug!("white_space", self.state);
        match self.state {
            Newline => {
                let depth = (&mut self.chars_loc)
                    .take_while_ref(|&(_,c)| c == '\t' || c == ' ')
                    .map(|(_,c)| match c {
                        ' '  => 1,
                        '\t' => 4,
                        _    => unreachable!()
                    })
                    .sum::<usize>() + delta;
                let indentation = self.indentation;
                self.state = Normal;
                debug!(format!("Indent:{} Depth:{}",indentation,depth));
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
            PartialName { ref start_loc, ref slices } => {
                let token = self.to_str_slices(start_loc, slices);
                self.state = Normal;
                Some(token)
            },
            Normal => self.next()
        }
    }

    fn move_kind(&mut self, start: usize, move_kind: MoveKind) -> Option<Token<'a>> {
        debug!("move_kind",self.state);
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
        debug!("set_partial",self.state);
        let was_newline = self.state == Newline;
        self.state = PartialName {
            start_loc: self.chars_loc.location,
            slices:    vec![name_part]
        };
        let indentation = self.indentation;
        if was_newline && indentation != 0 {
            self.indentation = 0;
            Some(Token::de_indent(self.chars_loc.location, indentation))
        } else {
            self.next()
        }
    }

    fn single_name(&mut self, name_part: NamePart<Slice>) -> Option<Token<'a>> {
        debug!("single_name",self.state);
        match self.state {
            PartialName { ref mut slices, .. } => {
                slices.push(name_part);
                self.next()
            },
            _ => self.set_partial(name_part)
        }
    }

    fn name(&mut self, index: usize) -> Option<Token<'a>> {
        debug!("name",self.state);
        use lexer::token::NamePart::*;
        match self.state {
            PartialName { ref mut slices, .. } => {
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
                match slices.last_mut() {
                    Some(name_part) => {
                        if let Ident(ref mut slice) = *name_part {
                            slice.end = index;
                        }
                    },
                    None => {
                        slices.push(Ident(Slice {
                            start: index,
                            end:   index
                        }));
                    }
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
        debug!("finalizer",self.state);
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
        self.token_stack.take().or_else(|| match self.chars_loc.next() {
            Some((index,c)) => match c {
                '#'  => Some(self.line_comment()),
                '\n' => Some(self.newline()),
                ' '  => self.white_space(1),
                '\t' => self.white_space(4),
                '>'  => self.move_kind(index,MoveKind::Copy),
                '-'  => self.move_kind(index,MoveKind::Link),
                '*'  => self.single_name(NamePart::Star),
                '&'  => self.single_name(NamePart::Ampersand),
                '/'  => self.single_name(NamePart::Slash),
                 _   => self.name(index)
            },
            None => self.finalizer()
        })
    }
}
