extern crate itertools;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub mod lexer;

use lexer::{Lex};

fn main() {
    let path = Path::new("/home/kyle/repos/spotfiles/config");
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display,
                           why.description()),
        Ok(file) => file
    };
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display,
                           why.description()),
        Ok(_) => ()
    };
    for token in &s.lex() {
        println!("{}",token)
    }
}
