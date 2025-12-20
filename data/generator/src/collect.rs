use crate::generator::constant::*;
use crate::generator::enumeration::*;
use crate::generator::module::*;
use crate::generator::table::*;
use crate::*;
use glob::glob;
use std::fs;
use std::path::{Path, PathBuf};

impl Generator {
    pub fn collect(&mut self) -> Result<(), Error> {
        println!("Collecting...");

        let base_module = ModuleEntry::new(Name::new("", Vec::new()));
        let mut next_modules = vec![base_module];

        while let Some(mut module) = next_modules.pop() {
            let child_modules = self.collect_module(&mut module, self.modules.len())?;

            for i in 0..child_modules.len() {
                module
                    .entries
                    .push(EntityEntry::ModuleIndex(self.modules.len() + i + 1));
            }

            next_modules.extend(child_modules);
            self.modules.push(module);
        }

        self.build_table_hierarchies()?;

        Ok(())
    }

    fn collect_module(&mut self, module: &mut ModuleEntry, index: usize) -> Result<Vec<ModuleEntry>, Error> {
        let module_dir = module.name.as_full_dir(&self.config.schema_dir);
        self.log(&format!("Collecting module `{}`", &module_dir.display()));

        // Collect entities
        let mut entity_files: Vec<PathBuf> = glob(module_dir.join("*.json").to_str().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        entity_files.sort();

        self.log(&format!("Found entity files: {:?}", &entity_files));

        for entity_file in entity_files {
            let file_name = entity_file.file_name().unwrap().to_str().unwrap();
            let components = file_name.split('.').collect::<Vec<_>>();

            if components.len() != 3 {
                return Err(Error::InvalidFileName(file_name.to_owned()));
            }

            let entity_name = module.name.get_child(components[0]);
            match components[1] {
                "table" => {
                    self.collect_table(module, &entity_file, entity_name)?;
                }
                "enum" => {
                    self.collect_enumeration(module, &entity_file, entity_name)?;
                }
                "const" => {
                    self.collect_constant(module, &entity_file, entity_name)?;
                }
                _ => {
                    return Err(Error::InvalidFileName(file_name.to_owned()));
                }
            }
        }

        // Collect child modules
        let mut child_modules = Vec::new();
        for entry in fs::read_dir(&module_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let dir_name = path.file_name().unwrap().to_str().unwrap();
            let module_name = module.name.get_child(dir_name);

            self.log(&format!("Found child directory `{}`", dir_name));
            child_modules.push(ModuleEntry::new(module_name));
        }
        child_modules.sort_by(|a, b| a.name.name.cmp(&b.name.name));

        Ok(child_modules)
    }

    fn collect_table(
        &mut self,
        module: &mut ModuleEntry,
        file: &Path,
        name: Name,
    ) -> Result<(), Error> {
        self.log(&format!("Collecting table `{}`", file.display()));

        let type_name = name.as_type(true);
        self.register_type(&type_name)?;

        let schema: TableSchema = serde_json::from_str(&fs::read_to_string(file)?)?;
        if !match &schema {
            TableSchema::Concrete(schema) => schema.enabled,
            TableSchema::Abstract(schema) => schema.enabled,
        } {
            return Ok(());
        }

        self.tables.push(TableEntry { name, schema });
        module
            .entries
            .push(EntityEntry::TableIndex(self.tables.len() - 1));

        self.table_indices.insert(type_name, self.tables.len() - 1);

        Ok(())
    }

    fn collect_enumeration(
        &mut self,
        module: &mut ModuleEntry,
        file: &Path,
        name: Name,
    ) -> Result<(), Error> {
        self.log(&format!("Collecting enumeration `{}`", file.display()));

        self.register_type(&name.as_type(true))?;

        let schema: EnumerationSchema = serde_json::from_str(&fs::read_to_string(file)?)?;

        self.enumerations.push(EnumerationEntry { name, schema });
        module
            .entries
            .push(EntityEntry::EnumerationIndex(self.enumerations.len() - 1));

        Ok(())
    }

    fn collect_constant(
        &mut self,
        module: &mut ModuleEntry,
        file: &Path,
        name: Name,
    ) -> Result<(), Error> {
        self.log(&format!("Collecting constant `{}`", file.display()));

        self.register_type(&name.as_type(true))?;

        let schema: ConstantSchema = serde_json::from_str(&fs::read_to_string(file)?)?;

        self.constants.push(ConstantEntry { name, schema });
        module
            .entries
            .push(EntityEntry::ConstantIndex(self.enumerations.len() - 1));

        Ok(())
    }

    fn register_type(&mut self, type_name: &str) -> Result<(), Error> {
        if self.names.contains(type_name) {
            return Err(Error::NamespaceCollision(type_name.to_owned()));
        }

        self.names.insert(type_name.to_owned());
        Ok(())
    }

    fn build_table_hierarchies(&mut self) -> Result<(), Error> {
        for (index, _) in self.tables.iter().enumerate() {
            self.table_hierarchies.insert(index, Vec::new());
        }

        for (index, table) in self.tables.iter().enumerate() {
            let extend = match table.schema.schematic().extend() {
                Some(extend) => extend,
                None => continue,
            };

            self.table_hierarchies
                .get_mut(&self.table_indices[extend])
                .unwrap()
                .push(index);
        }

        Ok(())
    }
}
