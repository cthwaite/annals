use std::collections::HashMap;
use group::Group;

#[derive(Debug, Default)]
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
