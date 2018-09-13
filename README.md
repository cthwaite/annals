# annals

annals is a library for procedurally generating text from simple grammars specified in YAML files, inspired by [tracery](https://github.com/galaxykate/tracery) and [improv](https://github.com/sequitur/improv).

Grammars take the form of YAML documents. The following example illustrates most of the features of `annals` grammar. Run the `illustrative` example to see it in action.

```yaml
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
    - "<@speaker>: <!animal>s! there's nothing quite like seeing <#1-100> wild <animal>s"
  - tags: { "size": "small" }
    rules:
    - "<@speaker>: oh look! a tiny <animal>!"
    - "<@speaker>: i just saw <#1-100> tiny <animal> babies at the zoo"
    - "<@speaker>: is that <(an !animal)>? yes, it's a tiny <animal>!"
```

## Rules

- `<name>` will expand to any `name`
- `<@name>` will expand a variable bound in the current Context
```rust
let mut ctx = Context::default();
ctx.bind("name", "foo")
```
- `<#1-100>` will expand to a number between 1 and 100
- `<!name>` will expand to `name`, and use the same value for any subsequent instance of `<name>` in the current rule
- `<(CMD ...)>` will execute a named command `CMD` to transform the output of the subsequently-specified rule. Available commands are currently limited to
  + "cap" or "capitalize"
  + "low" or "lowercase"
  + "title" or "titlecase"
  + "a" | "an" to prepend the indefinite article
  Note that these can be nested, so that <(title (a name))> would transform `a <name>` into titlecase.