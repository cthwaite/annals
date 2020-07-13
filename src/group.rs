use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AnnalsError;
use crate::rule::{rule_list, Rule};

fn always_false() -> bool {
    false
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Group {
    #[serde(default)]
    note: String,
    #[serde(default = "always_false")]
    bind: bool,
    #[serde(default)]
    pub tags: HashMap<String, String>,
    #[serde(with = "rule_list")]
    pub rules: Vec<Rule>,
}

impl Group {
    /// Create a new empty group of rules.
    pub fn new() -> Self {
        Group {
            note: String::new(),
            bind: false,
            tags: HashMap::new(),
            rules: vec![],
        }
    }

    /// Get the number of rules in the Group.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Check if the Group is empty.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Create a new `Group` from a slice of strings.
    ///
    /// # Arguments
    /// * `rules` - Slice of `String` or `&str` which will be parsed as `Rule`s.
    ///
    pub fn from_rules<T: AsRef<str>>(rules: &[T]) -> Result<Self, AnnalsError> {
        let rules: Result<Vec<_>, _> = rules.iter().map(|lit| Rule::new(lit.as_ref())).collect();
        let rules = rules?;
        Ok(Group {
            note: String::new(),
            bind: false,
            tags: HashMap::new(),
            rules,
        })
    }

    /// Add a rule to this group.
    /// # Arguments
    /// * `expr`: String slice to be parsed as a `Rule`.
    ///
    pub fn add_rule(&mut self, expr: &str) -> Result<(), AnnalsError> {
        let tmp_rule = Rule::new(expr)?;
        self.rules.push(tmp_rule);
        Ok(())
    }

    /// Add a list of rules to this Group.
    ///
    /// # Arguments
    /// * `rules` - Slice of `String` or `&str` which will be parsed as rules.
    ///
    pub fn add_rules<T: AsRef<str>>(&mut self, rules: &[T]) -> Result<(), AnnalsError> {
        let rules: Result<Vec<_>, _> = rules.iter().map(|lit| Rule::new(lit.as_ref())).collect();
        let rules = rules?;
        self.rules.extend(rules.into_iter());
        Ok(())
    }

    /// Set a tag for this group.
    ///
    /// # Arguments
    /// * `key` - Tag key.
    /// * `val` - Tag value.
    ///
    pub fn set_tag(&mut self, key: &str, val: &str) {
        self.tags.insert(key.to_string(), val.to_string());
    }
}

/// Iteration over each Rule in a Group.
pub struct GroupListIter<'a> {
    groups: Vec<&'a Group>,
    t_iter: ::std::slice::Iter<'a, Rule>,
    index: usize,
    pub size: usize,
}

impl<'a> GroupListIter<'a> {
    pub fn new(groups: Vec<&'a Group>) -> Self {
        let t_iter = groups.last().unwrap().rules.iter();
        let size = groups.iter().map(|grp| grp.rules.len()).sum();
        GroupListIter {
            groups,
            t_iter,
            index: 0,
            size,
        }
    }

    fn advance_group(&mut self) -> bool {
        self.groups.pop();
        match self.groups.last() {
            Some(group) => {
                self.t_iter = group.rules.iter();
                true
            }
            None => false,
        }
    }
}

impl<'a> Iterator for GroupListIter<'a> {
    type Item = (&'a Rule, &'a Group);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.t_iter.next() {
                Some(template) => {
                    self.index += 1;
                    return Some((&template, &self.groups.last().unwrap()));
                }
                None => {
                    if !self.advance_group() {
                        break;
                    }
                }
            }
        }
        None
    }
}
