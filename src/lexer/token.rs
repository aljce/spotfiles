use std::fmt::*;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum MoveKind {
    Link,
    Copy
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum NamePart<A> {
    Ident(A),
    Star,
    Ampersand,
    Slash
}

impl<'a> Display for NamePart<&'a str> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use self::NamePart::*;
        write!(f,"{}",match *self {
            Ident(ref ident) => ident,
            Star =>      "*",
            Ampersand => "&",
            Slash =>     "/"
        })
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum TokenKind<'a> {
    Indent,
    DeIndent,
    Newline,
    Move(MoveKind),
    Name(Vec<NamePart<&'a str>>)
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub struct Location {
    pub line:   usize,
    pub column: usize
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Length(pub usize);

impl Display for Length {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Length(len) => write!(f,"{}",len)
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Token<'a> {
    kind:     TokenKind<'a>,
    location: Location,
    length:   Length
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use self::TokenKind::*;
        use self::MoveKind::*;
        match self.kind {
           Indent     => write!(f,"INDENT {}",self.length),
           DeIndent   => write!(f,"DEINDENT {}",self.length),
           Newline    => write!(f,"NEWLINE"),
           Move(Link) => write!(f,"LINK"),
           Move(Copy) => write!(f,"COPY"),
           Name(ref nps) => {
               try!(write!(f,"<"));
               for (index,np) in nps.iter().enumerate() {
                   if index < nps.len() - 1 {
                       try!(write!(f,"{}@",np))
                   } else {
                       try!(write!(f,"{}",np))
                   }
               }
               write!(f,">")
           },
        }
    }
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
    pub fn name(loc: Location, input: Vec<NamePart<&'a str>>) -> Token<'a> {
        use self::NamePart::*;
        let len = input.iter().map(|np| {
            match *np {
                Ident(ident) => ident.len(),
                _ => 1
        }}).sum();
        Token {
            kind:     TokenKind::Name(input),
            location: loc,
            length:   Length(len)
        }
    }
}
