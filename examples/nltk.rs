extern crate annals;

use annals::{Scribe};

fn main() {
    let mut scribe = Scribe::new();
    scribe.load_cognates("texts/nltk.yml").unwrap();
    for _i in 0..10 {
        println!("{}", scribe.gen("S").unwrap());
    }
}
