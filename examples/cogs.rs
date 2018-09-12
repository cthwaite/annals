extern crate annals;
extern crate serde_yaml;

use annals::{Context, Scribe};


fn main() {
    let mut scribe = Scribe::new();
    scribe.load_cognates("texts/cogs.yml").unwrap();
    for _i in 0..5 {
        println!("{}", scribe.gen("root").unwrap());
    }
    println!("{}", scribe.gen("root").unwrap());
    let mut ctx = Context::default();
    ctx.set("colour", "red");
    println!("{}", scribe.gen_with("root", ctx).unwrap());
}
