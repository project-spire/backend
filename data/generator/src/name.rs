use std::path::{Path, PathBuf};
use heck::{ToUpperCamelCase, ToSnakeCase};

#[derive(Debug, Clone)]
pub struct Name {
    pub name: String,
    pub namespace: Vec<String>,
}

impl Name {
    pub fn new(name: &str, namespace: Vec<String>) -> Self {
        Self {
            name: name.to_owned(),
            namespace,
        }
    }

    pub fn get_child(&self, child_name: &str) -> Self {
        let mut child_namespace = self.namespace.clone();
        child_namespace.push(child_name.to_owned());

        Self::new(child_name, child_namespace)
    }

    pub fn parent_namespace(&self) -> &[String] {
        if self.namespace.is_empty() {
            &self.namespace
        } else {
            &self.namespace[..self.namespace.len() - 1]
        }
    }

    pub fn as_full_dir(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.namespace.join("/"))
    }

    pub fn as_entity(&self) -> String {
        self.name.to_snake_case()
    }

    pub fn as_type(&self, with_namespace: bool) -> String {
        if with_namespace && !self.namespace.is_empty() {
            let parent_namespace = self.parent_namespace().join("::");
            let parent_namespace = if parent_namespace.is_empty() {
                "".to_string()
            } else {
                format!("{}::", parent_namespace)
            };
            
            format!(
                "{}{}",
                parent_namespace,
                self.name.to_upper_camel_case(),
            )
        } else {
            self.name.to_upper_camel_case()
        }
    }
}