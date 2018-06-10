extern crate annals;
extern crate serde_yaml;

use annals::{Cognate, Context, Scribe};
use std::fs::File;

fn main() {
    let mut scribe = Scribe::new();
    scribe.load_cognates("texts/cogs.yml").unwrap();
    for _i in 0..5 {
        println!("{}", scribe.gen("greeting").unwrap());
    }
    println!("{}", scribe.gen("root").unwrap());
    let mut ctx = Context::new();
    ctx.set("colour", "red");
    println!("{}", scribe.gen_with("greeting", ctx).unwrap());
}
