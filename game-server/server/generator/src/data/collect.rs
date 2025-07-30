use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use serde::Deserialize;
use crate::data::{Generator, GenerateError, TableDef, ConstDef};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum EntityEntry {
    ModuleEntry {
        #[serde(rename = "mod")] file_path: String
    },
    TableEntry {
        #[serde(rename = "table")] file_path: String,
        #[serde(rename = "schema")] schema_path: String
    },
    ConstEntry {
        #[serde(rename = "const")] file_path: String
    },
}

impl Generator {
    pub fn collect(&mut self) -> Result<(), GenerateError> {
        println!("Collecting...");

        let base_module_path = self.config.data_dir.join("mod.json");
        let mut namespaces = VecDeque::new();

        self.do_collect(base_module_path, &mut namespaces)?;

        Ok(())
    }

    fn do_collect(
        &mut self,
        module_path: PathBuf,
        namespaces: &mut VecDeque<String>,
    ) -> Result<(), GenerateError> {
        println!("cargo:rerun-if-changed={}", module_path.display());

        // let components = module_path.file_name().unwrap().to_str().unwrap().split('.').collect::<Vec<_>>();
        // let current_dir: PathBuf = if module_path.file_stem().unwrap() == "mod" {
        //     module_path.parent().unwrap().into()
        // }  else {
        //     module_path.parent().unwrap().join(components[0])
        // };

        let entries: Vec<EntityEntry> = serde_json::from_str(
            &fs::read_to_string(&module_path)?
        )?;

        for entry in &entries {
            match entry {
                EntityEntry::ModuleEntry { file_path } => {
                    let components = file_path.split('.').collect::<Vec<_>>();
                    if components.len() != 3 || components[1] != "mod" || components[2] != "json" {
                        return Err(GenerateError::InvalidFile(format!("Invalid module file {file_path}")));
                    }
                    let namespace = components[0];

                    let next_module_path = self.full_data_path(namespaces, file_path);

                    namespaces.push_back(namespace.to_owned());
                    self.do_collect(next_module_path, namespaces)?;
                    namespaces.pop_back();
                }
                EntityEntry::TableEntry { file_path, schema_path } => {
                    let components = schema_path.split('.').collect::<Vec<&str>>();
                    if components.len() != 3 || components[1] != "table" || components[2] != "json" {
                        return Err(GenerateError::InvalidFile(format!("Invalid table schema file {schema_path}")));
                    }

                    let table_name = components[0];
                    let table_full_name = build_full_name(&namespaces, table_name);
                    if self.is_namespace_collision(&table_full_name) {
                        return Err(GenerateError::NamespaceCollision { name: table_full_name });
                    }

                    let schema_full_path = self.full_data_path(namespaces, schema_path);
                    println!("cargo:rerun-if-changed={}", schema_full_path.display());

                    let table_def = TableDef {
                        full_name: table_full_name.clone(),
                        file_path: self.full_data_path(namespaces, file_path),
                        schema_path: schema_full_path,
                    };
                    self.tables.insert(table_full_name, table_def);
                }
                EntityEntry::ConstEntry { file_path } => {
                    let components = file_path.split('.').collect::<Vec<&str>>();
                    if components.len() != 3 || components[1] != "const" || components[2] != "json" {
                        return Err(GenerateError::InvalidFile(format!("Invalid const file {file_path}")));
                    }

                    let const_full_name = build_full_name(&namespaces, components[0]);
                    if self.is_namespace_collision(&const_full_name) {
                        return Err(GenerateError::NamespaceCollision { name: const_full_name });
                    }

                    let const_full_path = self.full_data_path(namespaces, file_path);
                    println!("cargo:rerun-if-changed={}", const_full_path.display());

                    let const_def = ConstDef {
                        full_name: const_full_name.clone(),
                        file_path: const_full_path,
                    };
                    self.constants.insert(const_full_name, const_def);
                }
            }
        }

        Ok(())
    }

    fn is_namespace_collision(&self, name: &str) -> bool {
        self.tables.contains_key(name) || self.constants.contains_key(name)
    }
}

fn build_full_name(namespaces: &VecDeque<String>, name: &str) -> String {
    if namespaces.is_empty() {
        return name.to_owned();
    }
    format!("{}.{}",
        namespaces.iter().cloned().collect::<Vec<_>>().join("."),
        name
    )
}
