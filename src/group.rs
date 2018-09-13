use failure::Error;
use std::collections::HashMap;
use rule::{Rule, rule_list};

fn always_false() -> bool {
    false
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Group {
    #[serde(default)]
    note: String,
    #[serde(default="always_false")]
    bind: bool,
    #[serde(default)]
    pub tags: HashMap<String, String>,
    #[serde(with="rule_list")]
    pub rules: Vec<Rule>
}

impl Group {
    pub fn new() -> Self {
        Group {
            note: String::new(),
            bind: false,
            tags: HashMap::new(),
            rules: vec![]
        }
    }

    pub fn from_rules<T: AsRef<str>>(rules: &[T]) -> Result<Self, Error> {
        let rules : Result<Vec<_>, _> = rules.iter().map(|lit| Rule::new(lit.as_ref())).collect();
        let rules = rules.unwrap(); //rules?;
        Ok(Group {
            note: String::new(),
            bind: false,
            tags: HashMap::new(),
            rules
        })
    }

    pub fn add_rule(&mut self, expr: &str) -> Result<(), Error> {
        let tmp_rule = Rule::new(expr).unwrap();
        self.rules.push(tmp_rule);
        Ok(())
    }

    pub fn add_rules<T: AsRef<str>>(&mut self, rules: &[T]) -> Result<(), Error> {
        let rules : Result<Vec<_>, _> = rules.iter().map(|lit| Rule::new(lit.as_ref())).collect();
        let rules = rules.unwrap();
        self.rules.extend(rules.into_iter());
        Ok(())
    }

    pub fn set_tag(&mut self, key: &str, val: &str) {
        self.tags.insert(key.to_string(), val.to_string());
    }
}


pub struct GroupListIter<'a> {
    groups: Vec<&'a Group>,
    t_iter: ::std::slice::Iter<'a, Rule>,
    index: usize,
    pub size: usize
}

impl<'a> GroupListIter<'a> {
    pub fn new(groups: Vec<&'a Group>) -> Self {
        let t_iter = groups.last().unwrap().rules.iter();
        let size = groups.iter().map(|grp| grp.rules.len()).sum();
        GroupListIter {
            groups,
            t_iter,
            index: 0,
            size
        }
    }

    fn advance_group(&mut self) -> bool {
        self.groups.pop();
        match self.groups.last() {
            Some(group) => {
                self.t_iter = group.rules.iter();
                true
            },
            None => false
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
                },
                None => {
                    if !self.advance_group() {
                        break
                    }
                }
            }
        }
        None
    }
}

