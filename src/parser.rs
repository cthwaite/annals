#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("annals.pest");

#[derive(Parser)]
#[grammar = "annals.pest"]
pub struct AnnalsParser;


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simplest_template_property() {
        parses_to! {
            parser: AnnalsParser,
            input:  "<w>",
            rule:   Rule::template,
            tokens: [
                property(1, 2, [ident(1, 2)]),
            ]
        };
    }


    #[test]
    fn simple_template_property() {
        parses_to! {
            parser: AnnalsParser,
            input:  "<world>",
            rule:   Rule::template,
            tokens: [
                property(1, 6, [ident(1, 6)]),
            ]
        };
    }

    #[test]
    fn simple_template_literal_property() {
        parses_to! {
            parser: AnnalsParser,
            input:  "Hello <world>!",
            rule:   Rule::template,
            tokens: [
                literal(0, 6),
                property(7, 12, [ident(7, 12)]),
                literal(13, 14)
            ]
        };
    }

    #[test]
    fn simple_template_multi_property() {
        parses_to! {
            parser: AnnalsParser,
            input:  "Hello <new_world>, I'm from <other_planet>!",
            rule:   Rule::template,
            tokens: [
                literal(0, 6),
                property(7, 16, [ident(7, 16)]),
                literal(17, 28),
                property(29, 41, [ident(29, 41)]),
                literal(42, 43)
            ]
        };
    }
}
