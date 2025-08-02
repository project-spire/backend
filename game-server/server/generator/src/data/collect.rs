use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use serde::Deserialize;
use crate::data::{Generator, GenerateError, TableDef, ConstDef, ModuleDef, Entity, EnumDef};

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
    EnumEntry {
        #[serde(rename = "enum")] file_path: String
    },
}

impl Generator {
    pub fn collect(&mut self) -> Result<(), GenerateError> {
        println!("Collecting...");

        let mut namespaces = VecDeque::new();
        self.do_collect(self.config.base_module_path.clone(), &mut namespaces)?;

        Ok(())
    }

    fn do_collect(
        &mut self,
        module_path: PathBuf,
        namespaces: &mut VecDeque<String>,
    ) -> Result<(), GenerateError> {
        println!("cargo:rerun-if-changed={}", module_path.display());

        let entries: Vec<EntityEntry> = serde_json::from_str(
            &fs::read_to_string(&module_path)?
        )?;
        let mut entities = Vec::new();

        for entry in &entries {
            match entry {
                EntityEntry::ModuleEntry { file_path } => {
                    let namespace =  get_entity_name(file_path, "mod")?;
                    let next_module_path = self.full_data_path(namespaces, file_path);
                    
                    entities.push(Entity::Module(namespace.clone()));

                    namespaces.push_back(namespace);
                    self.do_collect(next_module_path, namespaces)?;
                    namespaces.pop_back();
                }
                EntityEntry::TableEntry { file_path, schema_path } => {
                    let table_name = get_entity_name(schema_path, "table")?;
                    let table_full_name = build_full_name(&namespaces, &table_name);
                    if self.is_namespace_collision(&table_full_name) {
                        return Err(GenerateError::NamespaceCollision { name: table_full_name });
                    }

                    let schema_full_path = self.full_data_path(namespaces, schema_path);
                    println!("cargo:rerun-if-changed={}", schema_full_path.display());

                    let table_def = TableDef {
                        namespaces: namespaces.clone().into(),
                        name: table_name.clone(),
                        file_path: self.full_data_path(namespaces, file_path),
                        schema_path: schema_full_path,
                    };
                    self.tables.insert(table_full_name, table_def);
                    
                    entities.push(Entity::Table(table_name));
                }
                EntityEntry::ConstEntry { file_path } => {
                    let const_name = get_entity_name(file_path, "const")?;
                    let const_full_name = build_full_name(&namespaces, &const_name);
                    if self.is_namespace_collision(&const_full_name) {
                        return Err(GenerateError::NamespaceCollision { name: const_full_name });
                    }

                    let const_full_path = self.full_data_path(namespaces, file_path);
                    println!("cargo:rerun-if-changed={}", const_full_path.display());

                    let const_def = ConstDef {
                        namespaces: namespaces.clone().into(),
                        name: const_name.clone(),
                        file_path: const_full_path,
                    };
                    self.constants.insert(const_full_name, const_def);
                    
                    entities.push(Entity::Const(const_name));
                },
                EntityEntry::EnumEntry { file_path } => {
                    let enum_name = get_entity_name(file_path, "enum")?;
                    let enum_full_name = build_full_name(&namespaces, &enum_name);
                    if self.is_namespace_collision(&enum_full_name) {
                        return Err(GenerateError::NamespaceCollision { name: enum_full_name });
                    }

                    let enum_full_path = self.full_data_path(namespaces, file_path);
                    println!("cargo:rerun-if-changed={}", enum_full_path.display());

                    let enum_def = EnumDef {
                        namespaces: namespaces.clone().into(),
                        name: enum_name.clone(),
                        file_path: enum_full_path,
                    };

                    self.enums.insert(enum_full_name, enum_def);

                    entities.push(Entity::Enum(enum_name));
                }
            }
        }

        self.modules.push(ModuleDef {
            name: get_entity_name(module_path.to_str().unwrap(), "mod")?,
            namespaces: namespaces.clone().into(),
            entities,
        });

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

fn get_entity_name(file_path: &str, expected_type: &str) -> Result<String, GenerateError> {
    let file_path = PathBuf::from(file_path);
    let file_name = file_path.file_name().unwrap().to_str().unwrap();

    let components = file_name.split('.').collect::<Vec<&str>>();
    if components.len() != 3 || components[1] != expected_type || components[2] != "json" {
        return Err(GenerateError::InvalidFile(format!("Invalid entity file {}", file_path.display())));
    }

    Ok(components[0].to_owned())
}
