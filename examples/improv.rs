extern crate annals;

use annals::{Context, Scribe};

fn main() {
    let mut scribe = Scribe::new();
    scribe.load_cognates("texts/improv.yml").unwrap();
    let mut bob = Context::new();
    bob.bind("name", String::from("Bob"));
    println!("{}", scribe.gen_with("root", bob).unwrap());

    let mut alice = Context::new();
    alice.bind("name", String::from("Alice"));
    alice.set("class", "mammal");
    println!("{}", scribe.gen_with("root", alice).unwrap());

    let mut carol = Context::new();
    carol.bind("name", String::from("Carol"));
    carol.set("class", "bird");
    println!("{}", scribe.gen_with("root", carol).unwrap());
}
