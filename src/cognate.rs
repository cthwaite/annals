use group::Group;
use failure::Error;
use std::slice::Iter;
use std::fs::File;
use serde_yaml;


/// Named collection of [`Group`](../group/struct.Group.html)s of
/// [`Rule`](../rule/struct.Rule.html)s.
#[derive(Debug, Serialize, Deserialize)]
pub struct Cognate {
    pub name: String,
    groups: Vec<Group>
}


impl Cognate {
    /// Create a new Cognate with the given name.
    ///
    /// # Arguments
    /// * `name` - A string representing the name for this Cognate.
    ///
    /// ```
    /// 
    /// use annals::cognate::Cognate;
    /// // Create a new Cognate named 'root'
    /// let cog = Cognate::new("root");
    /// ```
    pub fn new<S> (name: S) -> Self where S: Into<String> {
        Cognate {
            name: name.into(),
            groups: vec![]
        }
    }

    /// Get the number of Groups in the Cognate.
    pub fn len(&self) -> usize {
        self.groups.len()
    }

    /// Get the total number of Rules for each Group in the Cognate.
    pub fn rules_count(&self) -> usize {
        self.groups.iter().fold(0, |accu, curr| accu + curr.len())
    }

    /// Check if the `Cognate` contains any `Group`s.
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    /// Iterate over `Group`s in this `Cognate`.
    pub fn iter_groups(&self) -> Iter<Group> {
        self.groups.iter()
    }

    /// Create a new `Cognate` from a YAML file.
    ///
    /// # Arguments
    /// * `path` - Path to YAML file.
    ///
    /// ```rust,no_run
    /// use annals::cognate::Cognate;
    /// 
    /// let cog = Cognate::from_yaml("~/Documents/grammar.yml").unwrap();
    /// ```
    pub fn from_yaml(path: &str) -> Result<Self, Error> {
        let f = File::open(path)?;
        serde_yaml::from_reader(f).map_err(Into::into)
    }

    /// Create a new group from the passed slice of Rules. If successful, the
    /// new group will be immediately added to this Cognate.
    ///
    /// # Arguments
    /// * `rules` - Slice of `String` or `&str` which will be parsed as rules and
    /// inserted into a new `Group`.
    ///
    /// ```
    /// use annals::{cognate::Cognate, rule::Rule};
    /// 
    /// let mut cog = Cognate::new("root");
    /// assert_eq!(cog.len(), 0);
    /// assert_eq!(cog.rules_count(), 0);
    ///
    /// cog.group_from_rules(&["<A>", "A <B>"]).unwrap();
    ///
    /// assert_eq!(cog.len(), 1);
    /// assert_eq!(cog.rules_count(), 2);
    /// ```
    pub fn group_from_rules<T: AsRef<str>>(&mut self, rules: &[T]) -> Result<(), Error> {
        let grp = Group::from_rules(rules)?;
        self.groups.push(grp);
        Ok(())
    }

    /// Create a new Group within this Cognate, returning a mutable reference
    /// to the newly-created Group.
    ///
    /// ```
    /// use annals::cognate::Cognate;
    /// 
    /// let mut cog = Cognate::new("root");
    /// {
    ///     let mut grp = cog.add_group().unwrap();
    ///     grp.add_rule("<A>").unwrap();
    ///     grp.add_rule("A <B>").unwrap();
    /// }
    ///
    /// assert_eq!(cog.len(), 1);
    /// assert_eq!(cog.rules_count(), 2);
    /// ```
    pub fn add_group(&mut self) -> Option<&mut Group> {
        self.groups.push(Group::new());
        self.groups.last_mut()
    }
}
