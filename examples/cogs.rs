extern crate annals;
extern crate serde_yaml;

use annals::{Cognate, Context, Scribe};
use std::fs::File;

fn main() {
    let f = File::open("cogs.yml").unwrap();
    let cogs : Vec<Cognate> = serde_yaml::from_reader(f).unwrap();
    let mut scribe = Scribe::new();
    cogs.into_iter().for_each(|cog| scribe.insert_cognate(cog));
    for _i in 0..5 {
        println!("{}", scribe.gen("<greeting>").unwrap());
    }
    let mut ctx = Context::new();
    ctx.set("colour", "red");
    println!("{}", scribe.gen_with("greeting", ctx).unwrap());
}
