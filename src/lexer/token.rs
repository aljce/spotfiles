#[derive(Debug, PartialEq, PartialOrd)]
pub enum MoveKind {
    Link,
    Copy
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum TokenKind<'a> {
    Indent,
    DeIndent,
    Newline,
    Move(MoveKind),
    Name(&'a str)
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub struct Location {
    pub line: usize,
    pub column: usize
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Length(pub usize);

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Token<'a> {
    kind:     TokenKind<'a>,
    location: Location,
    length:   Length
}

impl<'a> Token<'a> {
    pub fn indent(loc: Location, depth: usize) -> Token<'a> {
        Token {
            kind:     TokenKind::Indent,
            location: loc,
            length:   Length(depth)
        }
    }
    pub fn de_indent(loc: Location, depth: usize) -> Token<'a> {
        Token {
            kind:     TokenKind::DeIndent,
            location: loc,
            length:   Length(depth)
        }
    }
    pub fn newline(loc: Location) -> Token<'a> {
        Token {
            kind:     TokenKind::Newline,
            location: loc,
            length:   Length(1)
        }
    }
    pub fn move_kind(loc: Location, move_k: MoveKind) -> Token<'a> {
        let len = match move_k {
            MoveKind::Link => 2,
            MoveKind::Copy => 1
        };
        Token {
            kind:     TokenKind::Move(move_k),
            location: loc,
            length:   Length(len)
        }
    }
    pub fn name(loc: Location, input: &'a str) -> Token<'a> {
        Token {
            kind:     TokenKind::Name(input),
            location: loc,
            length:   Length(input.len())
        }
    }
}
