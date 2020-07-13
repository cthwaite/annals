#[macro_use]
extern crate lazy_static;
// #[macro_use] extern crate serde_derive;

use std::collections::HashMap;
use std::fs::File;
use std::str::FromStr;

use rand::prelude::*;
use serde::{Deserialize, Serialize};
use titlecase::titlecase;

pub mod cognate;
pub mod context;
pub mod error;
pub mod group;
mod parse;
pub mod rule;

pub use context::Context;

use crate::cognate::Cognate;
use crate::error::AnnalsError;
use crate::group::GroupListIter;
use crate::parse::{Command, Token};
use crate::rule::Rule;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Scribe {
    cognates: HashMap<String, Cognate>,
}

impl Scribe {
    /// Create a new Scribe from a YAML file.
    pub fn from(path: &str) -> Result<Self, AnnalsError> {
        let f = File::open(path)?;
        serde_yaml::from_reader(f).map_err(Into::into)
    }

    /// Load a list of Cognates from a YAML file, inserting them into this Scribe.
    pub fn load_cognates(&mut self, path: &str) -> Result<(), AnnalsError> {
        let f = File::open(path)?;
        let cogs: Vec<Cognate> = serde_yaml::from_reader(f)?;
        for cog in cogs {
            self.insert_cognate(cog);
        }
        Ok(())
    }

    /// Load a list of Cognates from a YAML string, inserting them into this Scribe.
    pub fn load_cognates_str(&mut self, data: &str) -> Result<(), AnnalsError> {
        let cogs: Vec<Cognate> = serde_yaml::from_str(data)?;
        for cog in cogs {
            self.insert_cognate(cog);
        }
        Ok(())
    }

    /// Create and return a new Cognate.
    pub fn cognate(&mut self, name: &str) -> &mut Cognate {
        self.cognates
            .entry(name.to_string())
            .or_insert_with(|| Cognate::new(name))
    }

    /// Insert a Cognate.
    pub fn insert_cognate(&mut self, cognate: Cognate) {
        self.cognates.insert(cognate.name.to_string(), cognate);
    }

    /// Iterate over Cognates in this Scribe.
    pub fn iter(&self) -> std::collections::hash_map::Values<String, Cognate> {
        self.cognates.values()
    }

    /// Generate text from a named Cognate.
    pub fn gen(&self, cognate: &str) -> Result<String, AnnalsError> {
        let mut context = Context::default();
        let sel = self.select_rule(cognate, &mut context)?;
        self.expand_tokens(sel.tokens(), &mut context)
    }

    /// Generate text from a named Cognate using the passed Context.
    pub fn gen_with(&self, cognate: &str, mut context: Context) -> Result<String, AnnalsError> {
        let sel = self.select_rule(cognate, &mut context)?;
        self.expand_tokens(sel.tokens(), &mut context)
    }

    /// Generate text from the passed template string.
    pub fn expand(&self, rule: &str) -> Result<String, AnnalsError> {
        let new_rule = Rule::new(rule)?;
        let mut context = Context::default();
        self.expand_tokens(new_rule.tokens(), &mut context)
    }

    /// Generate text from the passed template string and Context.
    pub fn expand_with(&self, rule: &str, mut context: Context) -> Result<String, AnnalsError> {
        let new_rule = Rule::new(rule)?;
        self.expand_tokens(new_rule.tokens(), &mut context)
    }
    /// Save this Scribe to a YAML file.
    pub fn save(&self, path: &str) -> Result<(), AnnalsError> {
        let f = File::create(path)?;
        serde_yaml::to_writer(f, &self).map_err(Into::into)
    }

    /// Save this Scribe's Cognates to a YAML file.
    pub fn save_cognates(&self, path: &str) -> Result<(), AnnalsError> {
        let f = File::create(path)?;
        let cognates: Vec<&Cognate> = self.cognates.values().collect();
        serde_yaml::to_writer(f, &cognates).map_err(Into::into)
    }

    /// Select a template from a named Cognate using the passed Context.
    fn select_rule(&self, name: &str, context: &mut Context) -> Result<&Rule, AnnalsError> {
        match self.cognates.get(name) {
            Some(cognate) => {
                if cognate.is_empty() {
                    return Err(AnnalsError::EmptyCognate {
                        name: name.to_string(),
                    });
                }
                let groups = cognate
                    .iter_groups()
                    .filter(|grp| context.accept_strict(grp))
                    .collect::<Vec<_>>();
                if groups.is_empty() {
                    return Err(AnnalsError::NoSuitableGroups {
                        name: name.to_string(),
                        context: format!("{:?}", context.tags),
                    });
                }
                let mut templates = GroupListIter::new(groups);
                if templates.size == 0 {
                    return Err(AnnalsError::EmptyCognate {
                        name: name.to_string(),
                    });
                }
                let index = thread_rng().gen_range(0, templates.size);
                match templates.nth(index) {
                    Some(template) => {
                        context.merge_from_group(template.1);
                        Ok(template.0)
                    }
                    None => Err(AnnalsError::EmptyCognate {
                        name: name.to_string(),
                    }),
                }
            }
            None => Err(AnnalsError::UnknownCognate {
                name: name.to_string(),
            }),
        }
    }

    /// Expand an iterator over a sequence of Tokens into a String.
    #[inline]
    fn expand_tokens(
        &self,
        tokens: &[Token],
        context: &mut Context,
    ) -> Result<String, AnnalsError> {
        let ret = tokens
            .iter()
            .map(|tok| self.handle_token(tok, context))
            .collect::<Result<Vec<_>, AnnalsError>>()?
            .join("");
        Ok(ret)
    }

    fn expand_name(&self, name: &str, context: &mut Context) -> Result<String, AnnalsError> {
        context.descend();
        if let Some(bind) = context.get_binding(name) {
            return Ok(bind);
        }
        let sel = self.select_rule(name, context)?;
        let ret = self.expand_tokens(sel.tokens(), context);
        context.ascend();
        ret
    }

    /// Recursively expand a token to a String.
    fn handle_token(&self, token: &Token, context: &mut Context) -> Result<String, AnnalsError> {
        match token {
            Token::Literal(text) => Ok(text.clone()),
            Token::NonTerminal(name) => self.expand_name(name, context),
            Token::StickyNonTerminal(name) => self.expand_name(name, context).and_then(|ret| {
                context.bind(name, &ret);
                Ok(ret)
            }),
            Token::Binding(name) => {
                if let Some(bind) = context.get_binding(name) {
                    return Ok(bind);
                }
                Err(AnnalsError::UnboundVariable { name: name.clone() })
            }
            Token::Expression(cmd, token) => {
                match cmd {
                    Command::Capitalize => self.handle_token(token, context).and_then(|ret| {
                        let mut chs = ret.chars();
                        match chs.next() {
                            Some(t) => Ok(t.to_uppercase().chain(chs).collect()),
                            None => Ok("".to_string()),
                        }
                    }),
                    Command::Lowercase => self
                        .handle_token(token, context)
                        .and_then(|ret| Ok(ret.to_lowercase())),
                    Command::Titlecase => self
                        .handle_token(token, context)
                        .and_then(|ret| Ok(titlecase(&ret))),
                    Command::IndefiniteArticle => {
                        self.handle_token(token, context).and_then(|ret| {
                            match &ret.chars().next() {
                                // TODO: Stopgap; replace.
                                Some(ch) => match ch {
                                    'a' | 'e' | 'i' | 'o' | 'u' | 'A' | 'E' | 'I' | 'O' | 'U' => {
                                        Ok(format!("an {}", ret))
                                    }
                                    _ => Ok(format!("a {}", ret)),
                                },
                                None => Ok("".to_string()),
                            }
                        })
                    }
                }
            }
            Token::Range(lower, upper) => Ok(thread_rng().gen_range(*lower, *upper).to_string()),
            Token::VariableAssignment(name, bind) => {
                if context.get_binding(name).is_some() {
                    return Ok("".to_string());
                }
                let srule = self.select_rule(bind, context)?;
                let bind = self.expand_tokens(srule.tokens(), context)?;
                context.bind(name, &bind);
                let ret = self.expand_name(name, context);
                // TODO: exiting the 'scope' of a property, we drop the
                // property's bindings, but bindings, but it may be _optionally_
                // desirable to do so for tags as well.
                context.unbind(name);
                ret
            }
        }
    }
}

impl FromStr for Scribe {
    type Err = AnnalsError;

    /// Create a new Scribe from a YAML string.
    fn from_str(data: &str) -> Result<Self, AnnalsError> {
        serde_yaml::from_str(data).map_err(Into::into)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_handle_token() {
        let scr = Scribe::default();
        let mut ctx = Context::default();
        let tok = Token::Expression(
            Command::Titlecase,
            Box::new(Token::Literal("the duke of york".to_owned())),
        );
        let res = scr.handle_token(&tok, &mut ctx);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "The Duke of York".to_owned());
    }
}
