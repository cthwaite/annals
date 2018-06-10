extern crate annals;
extern crate serde_yaml;

use annals::Scribe;

fn main() {
    let mut scribe = Scribe::new();
    scribe.load_cognates("texts/tracery.yml").unwrap();
    for _i in 0..10 {
        println!("{}", scribe.gen("origin").unwrap());
    }
}
