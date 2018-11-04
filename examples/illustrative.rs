extern crate annals;
use annals::{Context, Scribe};

const YAML_STR : &str = r#"
---
- name: animal
  groups:
  - tags: { "size": "big" }
    rules:
    - elephant
    - whale
  - tags: { "size": "small" }
    rules:
    - mouse
    - milk snake
- name: expression
  groups:
  - rules:
    - "<@speaker>: what a beautiful <animal>"
    - "<@speaker>: oh, look at that <animal>!"
  - tags: { "size": "big" }
    rules:
    - "<@speaker>: wow! look at the size of that <animal>"
    - "<@speaker>: there's nothing quite like seeing <#1-100> wild <animal>s"
  - tags: { "size": "small" }
    rules:
    - "<@speaker>: oh look! a tiny <animal>!"
    - "<@speaker>: i just saw <#1-100> tiny <animal> babies at the zoo"
    - "<@speaker>: is that <(an !animal)>? yes, it's a tiny <animal>!"
"#;

fn main() {
    let mut scribe = Scribe::default();
    scribe.load_cognates_str(YAML_STR).unwrap();
    let mut ctx = Context::default();
    ctx.bind("speaker", "Bob");
    for _i in 0..10 {
        println!("{}", scribe.gen_with("expression", ctx.clone()).unwrap());
    }
}
