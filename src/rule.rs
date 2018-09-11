use std::fmt;
use std::slice::Iter;

use parse::{make_expr, Token};
use error::ParseError;


#[derive(Debug, PartialEq)]
pub struct Rule {
    literal: String,
    tokens: Vec<Token>,
    /* TODO: probability: i32 */
}


impl Rule {
    pub fn new(expr: &str) -> Result<Self, ParseError> {
        let literal = expr.into();
        let tokens = make_expr(expr)?;
        Ok(Rule {
            literal,
            tokens,
        })
    }

    pub fn from_string(literal: String) -> Result<Self, ParseError> {
        let tokens = make_expr(&literal)?;
        Ok(Rule {
            literal,
            tokens,
        })
    }

    /// Get the number of tokens in the Rule.
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Iterate over the tokens in the Rule.
    pub fn iter(&self) -> Iter<Token> {
        self.tokens.iter()
    }

    /// Get the literal expression the Rule is derived from.
    pub fn literal(&self) -> &str {
        &self.literal
    }

    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.literal)
    }
}

pub mod rule_list {
    use super::Rule;

    use serde::de::{Deserialize, Deserializer, Error};
    use serde::ser::{Serializer, SerializeSeq};

    pub fn serialize<S>(rules: &[Rule], serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(rules.len()))?;
        for rule in rules.iter() {
            seq.serialize_element(&rule.literal)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Rule>, D::Error>
        where D: Deserializer<'de> {
            let literals : Vec<String> = Vec::<String>::deserialize(deserializer)?;
            match literals.into_iter().map(Rule::from_string).collect() {
                Ok(rules) =>  Ok(rules),
                Err(e) => {
                    println!("{}", e);
                    Err(D::Error::custom(e))
                }
            }
    }
}

#[cfg(test)]
mod test {
    use super::Rule;
    #[test]
    fn test_template_valid() {
        macro_rules! good_rule {
            ( $literal: expr ) => {
                assert!(Rule::new($literal).is_ok())
            }
        }
        good_rule!("a");
        good_rule!("Hello");
        good_rule!("<w>");
        good_rule!("<a_snake>");
        good_rule!("Hello, <a_snake>!");
        good_rule!("Hello, <a_snake>, who is <disposition>!");
    }

    #[test]
    fn test_template_invalid() {
        macro_rules! bad_rule {
            ( $literal: expr ) => {
                assert!(Rule::new($literal).is_err())
            }
        }
        bad_rule!("");
        bad_rule!("<");
        bad_rule!(">");
        bad_rule!("Hello <!");
        bad_rule!("Hello >!");
        bad_rule!("<>");
        bad_rule!("<<>");
        bad_rule!("<><");
        bad_rule!("<><>");
        bad_rule!("<,>");
    }
}