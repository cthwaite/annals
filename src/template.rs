use failure::Error;
use pest::iterators::Pair;
use pest::Parser;

use error::AnnalsFailure;
use parser::{AnnalsParser, Rule};
use std::slice::Iter;


pub fn parse_token(token: Pair<Rule>) -> Token {
    match token.as_rule() {
        Rule::literal => Token::Literal(token.as_str().to_string()),
        Rule::property => Token::Property(token.into_inner().map(parse_token).collect()),
        Rule::ident => Token::Ident(token.as_str().to_string()),
        _ => Token::Unknown(token.as_str().to_string())
    }
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Literal(String),
    Property(Vec<Token>),
    Ident(String),
    Unknown(String)
}

#[derive(Debug, PartialEq)]
pub struct Template {
    pub literal: String,
    parsed: Vec<Token>
}


impl Template {
    pub fn new(text: &str) -> Result<Self, Error> {
        let parsed = Template::parse(text)?;
        Ok(Template {
            literal: text.to_string(),
            parsed
        })
    }

    pub fn from_string(literal: String) -> Result<Self, Error> {
        let parsed = Template::parse(&literal)?;
        Ok(Template {
            literal,
            parsed
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

    pub fn iter(&self) -> Iter<Token> {
        self.parsed.iter()
    }
}


pub mod template_list {
    use super::*;

    use serde::de::{Deserialize, Deserializer};
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
                Ok(templates) => Ok(templates),
                Err(_) => Ok(vec![])
            }
    }
}

mod test {
    use super::*;


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
        assert_eq!(*tmp.iter().next().unwrap(), Token::Property(vec![Token::Ident("a".to_owned())]));
    }
}
