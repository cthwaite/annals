extern crate annals;

use annals::{Context, Scribe};

fn main() {
    let mut scribe = Scribe::new();
    scribe.load_cognates("texts/improv.yml").unwrap();
    let mut bob = Context::default();
    bob.bind("name", "Bob");
    println!("{}", scribe.gen_with("root", bob).unwrap());

    let mut alice = Context::default();
    alice.bind("name", "Alice");
    alice.set("class", "mammal");
    println!("{}", scribe.gen_with("root", alice).unwrap());

    let mut carol = Context::default();
    carol.bind("name", "Carol");
    carol.set("class", "bird");
    println!("{}", scribe.gen_with("root", carol).unwrap());
}
