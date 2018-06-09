use failure::Error;
use std::collections::HashMap;
use template::{Template, template_list};


#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    #[serde(default)]
    pub tags: HashMap<String, String>,
    #[serde(with="template_list")]
    pub templates: Vec<Template>
}

impl Group {
    pub fn new() -> Self {
        Group {
            tags: HashMap::new(),
            templates: vec![]
        }
    }

    pub fn from_templates<T: AsRef<str>>(templates: &[T]) -> Result<Self, Error> {
        let templates : Result<Vec<_>, _> = templates.iter().map(|lit| Template::new(lit.as_ref())).collect();
        let templates = templates?;
        Ok(Group {
            tags: HashMap::new(),
            templates
        })
    }

    pub fn add_template(&mut self, template: &str) -> Result<(), Error> {
        let tmp = Template::new(template)?;
        self.templates.push(tmp);
        Ok(())
    }

    pub fn add_templates<T: AsRef<str>>(&mut self, templates: &[T]) -> Result<(), Error> {
        let templates : Result<Vec<_>, _> = templates.iter().map(|lit| Template::new(lit.as_ref())).collect();
        let templates = templates?;
        self.templates.extend(templates.into_iter());
        Ok(())
    }

    pub fn set_tag(&mut self, key: &str, val: &str) {
        self.tags.insert(key.to_string(), val.to_string());
    }
}


pub struct GroupListIter<'a> {
    groups: Vec<&'a Group>,
    t_iter: ::std::slice::Iter<'a, Template>,
    index: usize,
    pub size: usize
}

impl<'a> GroupListIter<'a> {
    pub fn new(groups: Vec<&'a Group>) -> Self {
        let t_iter = groups.last().unwrap().templates.iter();
        let size = groups.iter().map(|grp| grp.templates.len()).sum();
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
                self.t_iter = group.templates.iter();
                true
            },
            None => false
        }
    }
}


impl<'a> Iterator for GroupListIter<'a> {
    type Item = (&'a Template, &'a Group);

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

