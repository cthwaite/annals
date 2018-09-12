use group::Group;
use failure::Error;
use std::slice::Iter;
use std::fs::File;
use serde_yaml;


#[derive(Debug, Serialize, Deserialize)]
pub struct Cognate {
    pub name: String,
    groups: Vec<Group>
}


impl Cognate {
    pub fn new<S> (name: S) -> Self where S: Into<String> {
        Cognate {
            name: name.into(),
            groups: vec![]
        }
    }

    /// Check if the Cognate contains any Groups.
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    pub fn iter_groups(&self) -> Iter<Group> {
        self.groups.iter()
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
