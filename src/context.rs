use std::collections::HashMap;
use group::Group;
use std::collections::VecDeque;


#[derive(Debug, Default)]
pub struct Context {
    pub tags: HashMap<String, String>,
    bindings: HashMap<String, String>,
    unpop: VecDeque<Vec<String>>
}

impl Context {
    pub fn new(tags: HashMap<String, String>, bindings: HashMap<String, String>) -> Self {
        Context {
            tags,
            bindings,
            unpop: VecDeque::default(),
        }
    }

    /// Create a new Context with the given tags.
    pub fn with_tags(tags: HashMap<String, String>) -> Self {
        Context {
            tags,
            bindings: HashMap::default(),
            unpop: VecDeque::default(),
        }
    }

    /// Create a new Context with the given bindings.
    pub fn with_bindings(bindings: HashMap<String, String>) -> Self {
        Context {
            tags: HashMap::default(),
            bindings,
            unpop: VecDeque::default(),
        }
    }

    pub fn descend(&mut self) {
        self.unpop.push_back(vec![]);
    }

    pub fn ascend(&mut self) {
        match self.unpop.pop_back() {
            Some(vec) => {
                for name in vec {
                    self.unbind(name);
                }
            },
            None => ()
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
    pub fn bind<T: AsRef<str>>(&mut self, key: T, value: T) {
        self.bindings.insert(key.as_ref().to_string(), value.as_ref().to_string());
        match self.unpop.back_mut() {
            Some(vec) => vec.push(key.as_ref().to_string()),
            None => ()
        }
    }

    /// Remove a binding.
    pub fn unbind<T: AsRef<str>>(&mut self, key: T) {
        self.bindings.remove(key.as_ref());
    }

    /// Get the value for a binding, if any.
    pub fn get_binding(&mut self, key: &str) -> Option<String> {
        self.bindings.get(key).cloned()
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
