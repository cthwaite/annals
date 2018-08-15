use pest::iterators::Pair;
use pest::Parser;

use error::AnnalsFailure;
use parser::{AnnalsParser, Rule};



#[inline]
fn is_binding(token: &Token) -> bool {
    match token {
        Token::Binding(_) => true,
        _ => false
    }
}

#[inline]
pub fn expand_bound_property(token: Pair<Rule>) -> Token {
    let mut tokens = token.into_inner()
                      .map(parse_token)
                      .collect::<Vec<Token>>();
    let prop = tokens.pop().unwrap();
    match prop {
        Token::Ident(name) => {
            Token::PropertyWithBindings{
                binds: tokens.into_iter().map(|tok| {
                    match tok {
                        Token::Binding((key, val)) => (key, val),
                        _ => unreachable!()
                    }
                }).collect(),
                name
            }
        }
        _ => unreachable!()
    }
}

/// Translate pest-parsed string into Tokens.
pub fn parse_token(token: Pair<Rule>) -> Token {
    match token.as_rule() {
        Rule::literal => Token::Literal(token.as_str().to_string()),
        Rule::property => {
            let ident = token.into_inner().next().unwrap();
            Token::Property(ident.as_str().to_string())
        },
        Rule::bound_property => expand_bound_property(token),
        Rule::ident => Token::Ident(token.as_str().to_string()),
        Rule::variable => Token::Variable(token.as_str()[1..].to_string()),
        Rule::binding => {
            let mut pair = token.into_inner();
            let key = pair.next().unwrap().as_str().to_string();
            let val = pair.next().unwrap().as_str().to_string();
            Token::Binding((key, val))
        },
        Rule::range => {
            let mut pair = token.into_inner();
            let lower = pair.next().unwrap().as_str().to_string();
            let upper = pair.next().unwrap().as_str().to_string();
            Token::Range{ lower: lower.parse().unwrap(), upper: upper.parse().unwrap() }
        }
        _ => Token::Unknown(token.as_str().to_string())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Literal(String),
    Property(String),
    PropertyWithBindings{binds: Vec<(String, String)>, name: String},
    Ident(String),
    Range{lower: i32, upper: i32},
    Variable(String),
    Binding((String, String)),
    Unknown(String)
}

#[derive(Debug, PartialEq)]
pub struct Template {
    pub literal: String,
    pub tokens: Vec<Token>
}


impl Template {
    pub fn new(text: &str) -> Result<Self, AnnalsFailure> {
        let tokens = Template::parse(text)?;
        Ok(Template {
            literal: text.to_string(),
            tokens
        })
    }

    pub fn from_string(literal: String) -> Result<Self, AnnalsFailure> {
        let tokens = Template::parse(&literal)?;
        Ok(Template {
            literal,
            tokens
        })
    }

    pub fn parse(text: &str) -> Result<Vec<Token>, AnnalsFailure> {
        match AnnalsParser::parse(Rule::template, text) {
            Ok(pairs) => Ok(pairs.map(parse_token).collect::<Vec<Token>>()),
            Err(e) => Err(AnnalsFailure::InvalidTemplate{
                template: text.to_string(),
                error: format!("{}", e)
            })
        }
    }
}

pub mod template_list {
    use super::Template;

    use serde::de::{Deserialize, Deserializer, Error};
    use serde::ser::{Serializer, SerializeSeq};

    pub fn serialize<S>(templates: &[Template], serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(templates.len()))?;
        for template in templates.iter() {
            seq.serialize_element(&template.literal)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Template>, D::Error>
        where D: Deserializer<'de> {
            let literals : Vec<String> = Vec::<String>::deserialize(deserializer)?;
            match literals.into_iter().map(Template::from_string).collect() {
                Ok(templates) =>  Ok(templates),
                Err(e) => {
                    println!("{}", e);
                    Err(D::Error::custom(e))
                }
            }
    }
}

#[cfg(test)]
mod test {
    use super::{Template, Token};
    #[test]
    fn test_template_valid() {
        macro_rules! good_template {
            ( $literal: expr ) => {
                assert!(Template::new($literal).is_ok())
            }
        }
        good_template!("a");
        good_template!("Hello");
        good_template!("<w>");
        good_template!("<a_snake>");
        good_template!("Hello, <a_snake>!");
        good_template!("Hello, <a_snake>, who is <disposition>!");
    }

    #[test]
    fn test_template_invalid() {
        macro_rules! bad_template {
            ( $literal: expr ) => {
                assert!(Template::new($literal).is_err())
            }
        }
        bad_template!("");
        bad_template!("<");
        bad_template!(">");
        bad_template!("Hello <!");
        bad_template!("Hello >!");
        bad_template!("<>");
        bad_template!("<<>");
        bad_template!("<><");
        bad_template!("<><>");
        bad_template!("<,>");
    }

    #[test]
    fn test_template_iter() {
        let tmp = Template::new("<a>").unwrap();
        assert_eq!(tmp.tokens[0], Token::Property("a".to_owned()));
    }

    #[test]
    fn test_range() {
        let tmp = Template::new("<#39-100>").unwrap();
        assert_eq!(tmp.tokens[0], Token::Range{lower: 39, upper: 100});
    }
}
