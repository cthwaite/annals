#![allow(dead_code)]

#[macro_use] extern crate failure;
#[cfg(test)] #[macro_use] extern crate pest;
#[cfg(not(test))] extern crate pest;
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

    /// Create a new Cognate from a YAML file.
    pub fn from_yaml(path: &str) -> Result<Self, Error> {
        let f = File::open(path)?;
        serde_yaml::from_reader(f).map_err(Into::into)
    }

    /// Create a new group from the passed slice of Templates.
    pub fn group_from_templates<T: AsRef<str>>(&mut self, templates: &[T]) -> Result<(), Error> {
        let grp = Group::from_templates(templates)?;
        self.groups.push(grp);
        Ok(())
    }

    /// Add a new Group to this Cognate.
    pub fn add_group(&mut self) -> Option<&mut Group> {
        self.groups.push(Group::new());
        self.groups.last_mut()
    }
}

#[derive(Default)]
pub struct Context {
    pub tags: HashMap<String, String>,
    bindings: HashMap<String, String>
}

impl Context {
    pub fn new() -> Self {
        Context {
            tags: HashMap::default(),
            bindings: HashMap::default()
        }
    }

    /// Merge tags from a Group into this context.
    pub fn merge_from_group(&mut self, group: &Group) {
        for (key, value) in &group.tags {
            self.tags.insert(key.to_string(), value.to_string());
        }
    }

    /// Set the value of a tag.
    pub fn set<T: AsRef<str>>(&mut self, key: T, value: T) {
        self.tags.insert(key.as_ref().to_string(), value.as_ref().to_string());
    }

    /// Add a binding.
    pub fn bind(&mut self, key: &str, value: String) {
        self.bindings.insert(key.to_string(), value);
    }

    pub fn unbind<T: AsRef<str>>(&mut self, key: T) {
        self.bindings.remove(key.as_ref());
    }

    pub fn get_binding(&mut self, key: &str) -> Option<String> {
        if self.bindings.contains_key(key) {
            return Some(self.bindings[key].clone());
        }
        None
    }

    /// Check if a group's tags match the tags in this Context.
    pub fn accept(&self, group: &Group) -> bool {
        if self.tags.is_empty() || group.tags.is_empty() {
            return true;
        }
        !self.tags.iter()
                 .filter(|(ref key, _val)| group.tags.contains_key(*key))
                 .any(|(ref key, val)| group.tags[*key] != **val)

    }
}


#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Scribe {
    cognates: HashMap<String, Cognate>,
}

impl Scribe {
    pub fn new() -> Self {
        Scribe {
            cognates: HashMap::new(),
        }
    }

    /// Create a new Scribe from a YAML file.
    pub fn from(path: &str) -> Result<Self, Error> {
        let f = File::open(path)?;
        serde_yaml::from_reader(f).map_err(Into::into)
    }

    /// Create a new Scribe from a YAML string.
    pub fn from_str(data: &str) -> Result<Self, Error> {
        serde_yaml::from_str(data).map_err(Into::into)
    }

    /// Load a list of Cognates from a YAML file, inserting them into this Scribe.
    pub fn load_cognates(&mut self, path: &str) -> Result<(), Error> {
        let f = File::open(path)?;
        let cogs : Vec<Cognate> = serde_yaml::from_reader(f)?;
        for cog in cogs {
            self.insert_cognate(cog);
        }
        Ok(())
    }

    /// Load a list of Cognates from a YAML string, inserting them into this Scribe.
    pub fn load_cognates_str(&mut self, data: &str) -> Result<(), Error> {
        let cogs : Vec<Cognate> = serde_yaml::from_str(data)?;
        for cog in cogs {
            self.insert_cognate(cog);
        }
        Ok(())
    }

    /// Create and return a new Cognate.
    pub fn cognate(&mut self, name: &str) -> &mut Cognate {
        self.cognates.entry(name.to_string())
                     .or_insert_with(|| Cognate::new(name))
    }

    /// Insert a Cognate.
    pub fn insert_cognate(&mut self, cognate: Cognate) {
        self.cognates.insert(cognate.name.to_string(), cognate);
    }

    pub fn iter(&self) -> std::collections::hash_map::Values<String, Cognate> {
        self.cognates.values()
    }

    /// Generate text from a named Cognate.
    pub fn gen(&self, cognate: &str) -> Result<String, Error> {
        let mut context = Context::new();
        let template = self.select_template(cognate, &mut context)?;
        self.expand_tokens(template.iter(), &mut context)
    }

    /// Generate text from a named Cognate using the passed Context.
    pub fn gen_with(&self, cognate: &str, mut context: Context) -> Result<String, Error> {
        let template = self.select_template(cognate, &mut context)?;
        self.expand_tokens(template.iter(), &mut context)
    }

    /// Generate text from the passed template string.
    pub fn expand(&self, template: &str) -> Result<String, Error> {
        let tmp = Template::from_string(template.to_owned())?;
        let mut context = Context::new();
        self.expand_tokens(tmp.iter(), &mut context)
    }

    /// Generate text from the passed template string and Context.
    pub fn expand_with(&self, template: &str, mut context: Context) -> Result<String, Error> {
        let tmp = Template::from_string(template.to_owned())?;
        self.expand_tokens(tmp.iter(), &mut context)
    }
    /// Save this Scribe to a YAML file.
    pub fn save(&self, path: &str) -> Result<(), Error> {
        let f = File::create(path)?;
        serde_yaml::to_writer(f, &self).map_err(Into::into)
    }

    /// Save this Scribe's Cognates to a YAML file.
    pub fn save_cognates(&self, path: &str) -> Result<(), Error> {
        let f = File::create(path)?;
        let cognates : Vec<&Cognate> = self.cognates.values().collect();
        serde_yaml::to_writer(f, &cognates).map_err(Into::into)
    }
    /// Select a template from a named Cognate using the passed Context.
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
                if templates.size == 0 {
                    return Err(AnnalsFailure::EmptyCognate{name: name.to_string()}.into());
                }
                let index = thread_rng().gen_range(0, templates.size);
                match templates.nth(index) {
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

    /// Expand an iterator over a sequence of Tokens into a String.
    #[inline]
    fn expand_tokens(&self, tokens: Iter<Token>, context: &mut Context) -> Result<String, Error> {
        let expan = tokens
                        .map(|tok| self.handle_token(tok, context))
                        .collect::<Result<Vec<String>, Error>>()?;
        Ok(expan.join(""))
    }


    /// Recursively expand a token to a String.
    fn handle_token(&self, token: &Token, context: &mut Context) -> Result<String, Error> {
        match token {
            Token::Literal(text) => Ok(text.clone()),
            Token::PropertyWithBindings{binds, props} => {
                for (key, val) in binds.iter() {
                    if let Some(bind) = context.get_binding(key) {
                        context.bind(key, bind);
                    }
                    let temp = self.select_template(val, context)?;
                    let bind = self.expand_tokens(temp.iter(), context)?;
                    context.bind(key, bind);
                }
                let ret = self.expand_tokens(props.iter(), context);
                // exiting the 'scope' of a property, we drop the property's bindings, but
                // it may be _optionally_ desirable to do so for tags as well.
                binds.iter().for_each(|(key, _)| context.unbind(key));
                ret
            },
            Token::Property(tokens) => self.expand_tokens(tokens.iter(), context),
            Token::Ident(name) => {
                if let Some(bind) = context.get_binding(name) {
                    return Ok(bind);
                }
                let temp = self.select_template(name, context)?;
                self.expand_tokens(temp.iter(), context)
            },
            Token::Variable(name) => {
                if let Some(bind) = context.get_binding(name) {
                    return Ok(bind);
                }
                Err(AnnalsFailure::UnboundVariable{ name: name.clone() }.into())
            },
            Token::Binding(_) => unreachable!(),
            Token::Unknown(content) => Err(AnnalsFailure::UnknownToken{ content: content.to_string() }.into())
        }
    }
}
