#[macro_use]
extern crate nom;

pub use self::lexer::*;
pub mod lexer;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn main() {
    let path = Path::new("tmp");
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
    println!("{}",s);
    println!("{:?}",groups_p(s.as_bytes()))
}
