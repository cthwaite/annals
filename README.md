# annals

annals is a library for procedurally generating text from simple grammars, inspired by [tracery](https://github.com/galaxykate/tracery) and [improv](https://github.com/sequitur/improv).


## overview

annal uses *Templates* to generate text. each Template is a string:

```
i am a rule
i am also a rule
```

each Template may contain named references to other rules, wrapped in angle brackets.
these references, called *Properties*, are expanded recursively at runtime.

```
hello <world>
my favourite food is <food>
```

rules are bundled into *Groups*, which contain a list of rules, and an optional
set of key-value pairs incorporated into the text generation model

```yaml
# group for big animals
- tags: { "size": "big" }
  templates: ["elephant", "whale"]
# group for small animals
- tags: { "size": "small" }
  templates: ["mouse", "milk snake"]
```

*Groups* are bundled into named sets called *Cognates*; these are the names that
each Property refers to.

```yaml
- name: animal
  groups:
    - tags: { "size": "big" }
      templates: ["elephant", "whale"]
    - tags: { "size": "small" }
      templates: ["mouse", "milk snake"]
- name: expression
  groups:
    # this group will match any animal
    - templates: [
        "what a beautiful <animal>",
        "oh, look at that <animal>!"
    ]
    # this group will only match small animals
    - tags: { "size": "big" }
      templates: [ "wow! look at the size of that <animal>" ]
    # this group will only match big animals
    - tags: { "size": "small" }
      templates: [ "oh look! a tiny <animal>!" ]
```

finally, Cognates are bundled into a Scribe object, which is used to generate
text. to see the above grammar in action, run the `readme` example:

```
cargo run --example readme
```

## tracery-style bindings

like tracery, annals allows templates to be bound variables within the scope of a tag.

in the example below, the Properties `hero` and `heroPet` are variables, mapped
to the templates `name` and `animal` within the scope of the `story` property.
when the `story` Property is expanded, `hero` and `heroPet` are first assigned
values by fully expanding their respective templates, after which the `story`
template is expanded, substituting the variables in where appropriate.

```yaml
- name: name
  groups:
    - templates: ["Arjun","Yuuma","Darcy","Mia","Chiaki","Izzi","Azra","Lina"]
- name: animal
  groups:
    - templates: ["unicorn","raven","sparrow","scorpion","coyote","eagle","owl","lizard","zebra","duck","kitten"]
- name: mood
  groups:
    - templates: ["vexed","indignant","impassioned","wistful","astute","courteous"]
- name: story
  groups:
    - templates:
        - "<hero> traveled with her pet <heroPet>. <hero> was never <mood>, for the <heroPet> was always too <mood>."
- name: origin
  groups:
    - templates:
        - "<[hero:name][heroPet:animal]story>"
```

to see the above grammar in action, run the `tracery` example:

```
cargo run --example tracery
```

## improv-style models

like improv, annals uses models to direct output.

```yaml
- name: animal
  groups:
    - tags: { "class": "mammal" }
      templates:
        - dog
        - cat
    - tags: { "class": "bird" }
      templates:
        - parrot
- name: root
  groups:
    - templates:
        - <name>: I have a <animal> who is <#2-7> years old.
```

```rust
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
```

to see the above grammar in action, run the `improv` example:

```
cargo run --example improv
```

## todo
- full suite of tests
- comprehensive documentation
- builtin functions for inline articles and capitalisation
- builder model for Context and Cognate
