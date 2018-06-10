extern crate annals;
use annals::Scribe;

const YAML_STR : &str = r#"
---
- name: animal
  groups:
    - tags: { "size": "big" }
      templates: ["elephant", "whale"]
    - tags: { "size": "small" }
      templates: ["mouse", "milk snake"]
- name: expression
  groups:
    - templates: [
        "ANY: what a beautiful <animal>",
        "ANY: oh, look at that <animal>!"
    ]
    - tags: { "size": "big" }
      templates: [ "BIG: wow! look at the size of that <animal>" ]
    - tags: { "size": "small" }
      templates: [ "SMALL: oh look! a tiny <animal>!" ]"#;

fn main() {
    let mut scribe = Scribe::new();
    scribe.load_cognates_str(YAML_STR).unwrap();
    for _i in 0..10 {
        println!("{}", scribe.gen("expression").unwrap());
    }
}
