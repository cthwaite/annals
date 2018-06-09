#![allow(dead_code)]

#[macro_use] extern crate failure;
#[macro_use] extern crate pest;
#[macro_use] extern crate pest_derive;
extern crate rand;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_yaml;

use failure::Error;
use rand::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::slice::Iter;


mod error;
mod group;
mod parser;
mod template;

use error::AnnalsFailure;
use group::{Group, GroupListIter};
use template::{Token, Template};


#[derive(Debug, Serialize, Deserialize)]
pub struct Cognate {
    name: String,
    groups: Vec<Group>
}

impl Cognate {
    pub fn new<S> (name: S) -> Self where S: Into<String> {
        Cognate {
            name: name.into(),
            groups: vec![]
        }
    }

    pub fn from_yaml(path: &str) -> Result<Self, Error> {
        let f = File::open(path)?;
        serde_yaml::from_reader(f).map_err(Into::into)
    }

    pub fn group_from_templates<T: AsRef<str>>(&mut self, templates: &[T]) -> Result<(), Error> {
        let grp = Group::from_templates(templates)?;
        self.groups.push(grp);
        Ok(())
    }

    pub fn add_group(&mut self) -> Option<&mut Group> {
        self.groups.push(Group::new());
        self.groups.last_mut()
    }
}

pub struct Context {
    pub tags: HashMap<String, String>
}

impl Context {
    pub fn new() -> Self {
        Context {
            tags: HashMap::new()
        }
    }

    pub fn merge_from_group(&mut self, group: &Group) {
        for (key, value) in &group.tags {
            self.tags.insert(key.to_string(), value.to_string());
        }
    }

    pub fn set<T: AsRef<str>>(&mut self, key: T, value: T) {
        self.tags.insert(key.as_ref().to_string(), value.as_ref().to_string());
    }

    pub fn accept(&self, group: &Group) -> bool {
        if self.tags.is_empty() || group.tags.is_empty() {
            return true;
        }
        !self.tags.iter()
                 .filter(|(ref key, _val)| group.tags.contains_key(*key))
                 .any(|(ref key, val)| group.tags[*key] != **val)

    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Scribe {
    filters: Vec<String>,
    cognates: HashMap<String, Cognate>,
}
impl Scribe {
    pub fn new() -> Self {
        Scribe {
            filters: vec![],
            cognates: HashMap::new(),
        }
    }

    pub fn from_yaml(path: &str) -> Result<Self, Error> {
        let f = File::open(path)?;
        serde_yaml::from_reader(f).map_err(Into::into)
    }

    pub fn cognate(&mut self, name: &str) -> &mut Cognate {
        self.cognates.entry(name.to_string()).or_insert_with(|| Cognate::new(name))
    }

    pub fn insert_cognate(&mut self, cognate: Cognate) {
        self.cognates.insert(cognate.name.to_string(), cognate);
    }

    fn select_template(&self, name: &str, context: &mut Context) -> Result<&Template, Error> {
        match self.cognates.get(name) {
            Some(cognate) => {
                if cognate.groups.is_empty() {
                    return Err(AnnalsFailure::EmptyCognate{name: name.to_string()}.into());
                }
                let groups = cognate.groups.iter()
                                            .filter(|grp| context.accept(grp))
                                            .collect::<Vec<_>>();
                if groups.is_empty() {
                    let context = format!("{:?}", context.tags);
                    return Err(AnnalsFailure::NoSuitableGroups{context}.into());
                }
                let mut templates = GroupListIter::new(groups);
                let max = templates.size;
                match templates.nth(thread_rng().gen_range(0, max)) {
                    Some(template) => {
                        context.merge_from_group(template.1);
                        Ok(template.0)
                    },
                    None => {
                        Err(AnnalsFailure::EmptyCognate{name: name.to_string()}.into())
                    }
                }
            },
            None => Err(AnnalsFailure::UnknownCognate{name: name.to_string()}.into())
        }
    }

    #[inline]
    fn expand_tokens(&self, tokens: Iter<Token>, context: &mut Context) -> Result<String, Error> {
        let expan = tokens
                        .map(|tok| self.handle_token(tok, context))
                        .collect::<Result<Vec<String>, Error>>()?;
        Ok(expan.join(""))
    }

    fn handle_token(&self, token: &Token, context: &mut Context) -> Result<String, Error> {
        match token {
            Token::Literal(text) => Ok(text.clone()),
            Token::Property(tokens) => self.expand_tokens(tokens.iter(), context),
            Token::Ident(name) => {
                let temp = self.select_template(name, context)?;
                self.expand_tokens(temp.iter(), context)
            },
            Token::Unknown(_) => Ok("???".to_owned())
        }
    }

    pub fn gen(&self, cognate: &str) -> Result<String, Error> {
        let mut context = Context::new();
        let template = self.select_template(cognate, &mut context)?;
        self.expand_tokens(template.iter(), &mut context)
    }

    pub fn gen_with(&self, cognate: &str, mut context: Context) -> Result<String, Error> {
        let template = self.select_template(cognate, &mut context)?;
        self.expand_tokens(template.iter(), &mut context)
    }

    pub fn gen_from(&self, template: &str) -> Result<String, Error> {
        let tmp = Template::from_string(template.to_owned())?;
        let mut context = Context::new();
        self.expand_tokens(tmp.iter(), &mut context)
    }

    /// Load a list of Cognates from a YAML file, inserting them into this Scribe.
    pub fn load_cognates(&mut self, path: &str) -> Result<(), Error> {
        let f = File::create(path)?;
        let cogs : Vec<Cognate> = serde_yaml::from_reader(f)?;
        for cog in cogs {
            self.insert_cognate(cog);
        }
        Ok(())
    }

    /// Save
    pub fn save(&self, path: &str) -> Result<(), Error> {
        let f = File::create(path)?;
        serde_yaml::to_writer(f, &self).map_err(Into::into)
    }

    /// Save stored cognates to a YAML file.
    pub fn save_cognates(&self, path: &str) -> Result<(), Error> {
        let f = File::create(path)?;
        let cognates : Vec<&Cognate> = self.cognates.values().collect();
        serde_yaml::to_writer(f, &cognates).map_err(Into::into)
    }
}
