use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::PathBuf;
use serde::Deserialize;
use crate::data::*;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum EntityDeclaration {
    Module {
        #[serde(rename = "mod")] schema_filename: String
    },
    Table {
        #[serde(rename = "table")] table_filename: String,
        #[serde(rename = "schema")] schema_filename: String
    },
    Enumeration {
        #[serde(rename = "enum")] schema_filename: String
    },
    Constant {
        #[serde(rename = "const")] schema_filename: String
    },
}

impl EntityDeclaration {
    fn validate(&self) -> Result<(), GenerateError> {
        fn check_schema_filename<'a>(
            filename: &'a str,
            expected_entity: &str
        ) -> Result<&'a str, GenerateError> {
            let components = filename.split('.').collect::<Vec<_>>();
            if components.len() != 3
                || components[1] != expected_entity
                || components[2] != "json" {
                return Err(GenerateError::InvalidFileName(filename.to_owned()));
            }
            Ok(components[0])
        }

        fn check_table_filename(filename: &str) -> Result<&str, GenerateError> {
            let components = filename.split('.').collect::<Vec<_>>();
            if components.len() != 2
                || components[1] != "ods" {
                return Err(GenerateError::InvalidFileName(filename.to_owned()));
            }
            Ok(components[0])
        }

        match self {
            EntityDeclaration::Module { schema_filename } => {
                check_schema_filename(schema_filename, "mod")?;
            },
            EntityDeclaration::Table { table_filename, schema_filename } => {
                let name1 = check_table_filename(table_filename)?;
                let name2 = check_schema_filename(schema_filename, "table")?;
                if name1 != name2 {
                    return Err(GenerateError::InvalidFileName(format!(
                        "{table_filename}, {schema_filename}"
                    )));
                }
            },
            EntityDeclaration::Enumeration { schema_filename } => {
                check_schema_filename(schema_filename, "enum")?;
            },
            EntityDeclaration::Constant { schema_filename } => {
                check_schema_filename(schema_filename, "const")?;
            },
        }
        Ok(())
    }

    fn get_name(&self, namespaces: &[String]) -> Name {
        fn extract_name(filename: &str) -> String {
            let components = filename.split('.').collect::<Vec<_>>();
            components[0].to_owned()
        }

        let name = match self {
            EntityDeclaration::Module { schema_filename } => {
                extract_name(schema_filename)
            },
            EntityDeclaration::Table { table_filename: _, schema_filename } => {
                extract_name(schema_filename)
            },
            EntityDeclaration::Enumeration { schema_filename } => {
                extract_name(schema_filename)
            },
            EntityDeclaration::Constant { schema_filename } => {
                extract_name(schema_filename)
            },
        };
        Name::new(&name, &namespaces)
    }
}

impl Generator {
    pub fn collect(&mut self) -> Result<(), GenerateError> {
        println!("Collecting...");

        let mut collect_tasks = vec![(
            ModuleEntry::new(Name::new(
                "data",
                &Vec::new(),
            )),
            self.config.base_module_path.clone(),
            Vec::new(),
        )];

        while let Some((mut module_entry, module_path, namespaces)) = collect_tasks.pop() {
            println!("cargo:rerun-if-changed={}", module_path.display());

            let declarations: Vec<EntityDeclaration> = serde_json::from_str(
                &fs::read_to_string(&module_path)?
            )?;
            for declaration in &declarations {
                self.do_collect(&mut collect_tasks, &mut module_entry, &namespaces, declaration)?;
            }

            self.modules.push(module_entry);
        }

        self.base_module_index = Some(0);

        self.build_dependency_levels()?;

        Ok(())
    }

    fn do_collect(
        &mut self,
        collect_tasks: &mut Vec<(ModuleEntry, PathBuf, Vec<String>)>,
        module_entry: &mut ModuleEntry,
        namespaces: &Vec<String>,
        declaration: &EntityDeclaration,
    ) -> Result<(), GenerateError> {
        declaration.validate()?;
        let name = declaration.get_name(&namespaces);

        match declaration {
            EntityDeclaration::Module { schema_filename } => {
                let child_module_path = self.full_data_path(&namespaces, schema_filename);
                let child_module_entry = ModuleEntry {
                    name: name.clone(),
                    entries: Vec::new(),
                };

                let mut namespaces = namespaces.clone();
                namespaces.push(name.as_entity());
                collect_tasks.push((child_module_entry, child_module_path, namespaces));

                module_entry.entries.push(EntityEntry::ModuleIndex(
                    self.modules.len() + 1
                ))
            },
            EntityDeclaration::Table { table_filename, schema_filename } => {
                let type_name = name.as_type(true);
                self.register_type(type_name.clone())?;

                let schema_full_path = self.full_data_path(&namespaces, schema_filename);
                println!("cargo:rerun-if-changed={}", schema_full_path.display());

                let schema: TableSchema = serde_json::from_str(
                    &fs::read_to_string(&schema_full_path)?
                )?;
                
                self.tables.push(TableEntry {
                    name,
                    table_path: PathBuf::from(namespaces.join("/")).join(table_filename),
                    schema,
                });
                module_entry.entries.push(EntityEntry::TableIndex(
                    self.tables.len() - 1
                ));

                self.table_indices.insert(type_name, self.tables.len() - 1);
            },
            EntityDeclaration::Enumeration { schema_filename } => {
                self.register_type(name.as_type(true))?;

                let schema_full_path = self.full_data_path(&namespaces, schema_filename);
                println!("cargo:rerun-if-changed={}", schema_full_path.display());

                let schema: EnumerationSchema = serde_json::from_str(
                    &fs::read_to_string(&schema_full_path)?
                )?;

                self.enumerations.push(EnumerationEntry {
                    name,
                    schema,
                });
                module_entry.entries.push(EntityEntry::EnumerationIndex(
                    self.enumerations.len() - 1
                ));
            },
            EntityDeclaration::Constant { schema_filename } => {
                self.register_type(name.as_type(true))?;

                let schema_full_path = self.full_data_path(&namespaces, schema_filename);
                println!("cargo:rerun-if-changed={}", schema_full_path.display());

                let schema: ConstantSchema = serde_json::from_str(
                    &fs::read_to_string(&schema_full_path)?
                )?;

                self.constants.push(ConstantEntry {
                    name,
                    schema,
                });
                module_entry.entries.push(EntityEntry::ConstantIndex(
                    self.constants.len() - 1
                ));
            },
        }

        Ok(())
    }

    fn register_type(&mut self, typename: String) -> Result<(), GenerateError> {
        if self.type_names.contains(&typename) {
            return Err(GenerateError::NamespaceCollision(typename));
        }

        self.type_names.insert(typename);
        Ok(())
    }

    fn build_dependency_levels(&mut self) -> Result<(), GenerateError> {
        // 1. Build graph
        let mut graph: HashMap<usize, Vec<usize>> = HashMap::new();
        for (index, table) in self.tables.iter().enumerate() {
            graph.insert(index, Vec::new());

            for field in &table.schema.fields {
                let link_type = match &field.kind {
                    FieldKind::Link { link_type } => link_type,
                    _ => continue,
                };

                let target_index = match self.table_indices.get(link_type) {
                    Some(index) => *index,
                    None => {
                        return Err(GenerateError::UnknownType(link_type.clone()));
                    },
                };

                graph.get_mut(&index).unwrap().push(target_index);
            }
        }

        // 2. Build in-degrees
        let mut in_degrees: HashMap<usize, usize> = graph
            .iter()
            .map(|(node, _)| (*node, 0))
            .collect();

        for nodes in graph.values() {
            for node in nodes {
                *in_degrees.get_mut(node).unwrap() += 1;
            }
        }

        // 3. Sort topologically (Khan's Algorithm)
        let mut queue: VecDeque<usize> = in_degrees
            .iter()
            .filter(|&(_, &degree)| degree == 0)
            .map(|(node, _)| *node)
            .collect();
        let mut sorted_count = 0;

        while !queue.is_empty() {
            let level_size = queue.len();
            let mut current_level = Vec::with_capacity(level_size);

            for _ in 0..level_size {
                let u = queue.pop_front().unwrap();
                for v in graph.get(&u).unwrap() {
                    let degree = in_degrees.get_mut(v).unwrap();
                    *degree -= 1;

                    if *degree == 0 {
                        queue.push_back(*v);
                    }
                }

                current_level.push(u);
                sorted_count += 1;
            }
            self.dependency_levels.push(current_level);
        }

        // Cycle detected
        if sorted_count != graph.len() {
            return Err(GenerateError::CircularDependency)
        }

        Ok(())
    }
}
