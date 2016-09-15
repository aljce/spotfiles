use nom::IResult;

#[derive(Debug)]
pub struct Title<'a> {
    name: &'a [u8],
    groups: Vec<&'a [u8]>
}

enum MoveKind {
    Link,
    Copy
}

#[derive(Debug)]
pub enum Move<'a> {
    Link(&'a [u8],&'a [u8]),
    Copy(&'a [u8],&'a [u8])
}

#[derive(Debug)]
pub struct Group<'a> {
    title: Title<'a>,
    moves: Vec<Move<'a>>
}

pub enum ParseError {
    UnsupportedMoveType
}

fn is_whitespace(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\r'
}

fn isnt_whitespace(c: u8) -> bool {
    !is_whitespace(c) && c != b'\n'
}

named!(name_p,
       take_while1!(isnt_whitespace)
);

named!(whitespace_p,
       take_while1!(is_whitespace)
);

named!(title_p<&[u8], Title>, chain!(
    name:   name_p ~
            whitespace_p ~
    groups: separated_list!(whitespace_p, name_p ),
    || {Title{name: name, groups: groups}}
));

// named!(move_kind_p, error!(ParseError::UnsupportedMoveType, alt_complete!(
//     value!(MoveKind::Link, tag_s!("->")) |
//     value!(MoveKind::Copy, tag_s!(">"))
// )));
named!(move_kind_p<&[u8],MoveKind>, alt_complete!(
    value!(MoveKind::Link, tag!("->")) |
    value!(MoveKind::Copy, tag!(">"))
));

named!(move_p<&[u8], Move> ,chain!(
    file:      name_p ~
               whitespace_p ~
    move_kind: move_kind_p ~
               whitespace_p ~
    location:  name_p,
    || {
        match move_kind {
            MoveKind::Link => Move::Link(file,location),
            MoveKind::Copy => Move::Copy(file,location),
        }
    }
));

named!(group_p<&[u8], Group> , chain!(
    title: title_p ~
           tag!("\n") ~
    moves: terminated!(
             many0!(preceded!(whitespace_p, move_p)),
             tag!("\n")
           ),
    || {Group{title: title, moves: moves}}
));

pub fn groups_p<'a>(input: &'a [u8]) -> IResult<&'a [u8], Vec<Group<'a>>> {
    complete!(input, many0!(group_p))
}
