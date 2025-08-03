use std::fs;
use crate::data::*;
use crate::data::generate::*;

impl Generator {
    pub fn generate_module(
        &self,
        module: &ModuleEntry
    ) -> Result<(), GenerateError> {
        let module_dir = self.full_gen_dir(&module.name.namespaces);
        fs::create_dir_all(&module_dir)?;

        let mut code = self.generate_module_code(module)?;

        if module.name.as_entity() == "data" && module.name.namespaces.is_empty() {
            // Append base module code
            let base_code: String = self.generate_base_module_code(module)?;
            code.push_str(&base_code);
        } else {
            // Create children directory
            let mut namespaces = module.name.namespaces.clone();
            namespaces.push(module.name.as_entity());

            let children_dir = self.full_gen_dir(&namespaces);
            fs::create_dir_all(&children_dir)?;
        }

        fs::write(
            module_dir.join(format!("{}.rs", module.name.as_entity())),
            code,
        )?;

        Ok(())
    }

    fn generate_module_code(&self, module: &ModuleEntry) -> Result<String, GenerateError> {
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        for entry in &module.entries { match entry {
            EntityEntry::ModuleIndex(index) => {
                let child = &self.modules[*index];
                imports.push(format!("pub mod {};", child.name.as_entity()));
            },
            EntityEntry::TableIndex(index) => {
                let child = &self.tables[*index];
                let name = &child.name;

                imports.push(format!("pub mod {};", name.as_entity()));
                exports.push(format!(
                    "pub use {}::{{{}, {}, {}}};",
                    name.as_entity(),
                    name.as_type(false),
                    name.as_data_type(),
                    name.as_data_type_cell(),
                ));
            },
            EntityEntry::ConstantIndex(index) => {
                let child = &self.constants[*index];
                imports.push(format!("pub mod {};", child.name.as_entity()));
            },
            EntityEntry::EnumerationIndex(index) => {
                let child = &self.enumerations[*index];
                let name = &child.name;

                imports.push(format!("pub mod {};", name.as_entity()));
                exports.push(format!(
                    "pub use {}::{};",
                    name.as_entity(),
                    name.as_type(false),
                ));
            }, }
        }

        let imports_code = imports.join("\n");
        let exports_code = exports.join("\n");

        Ok(format!(
r#"{GENERATED_FILE_WARNING}
{imports_code}

{exports_code}
"#
        ))
    }

    fn generate_base_module_code(
        &self,
        _module: &ModuleEntry,
    ) -> Result<String, GenerateError> {
        let mut level_handles = Vec::new();
        for (level, indices) in self.dependency_levels.iter().enumerate() {
            if indices.is_empty() {
                continue;
            }

            let mut handles = Vec::new();
            let handles_name = format!("level_{level}_handles");

            for index in indices {
                let table = &self.tables[*index];
                let name = &table.name;

                handles.push(format!(
                    "{TAB}add::<{CRATE_PREFIX}::{}::{}>(&data_dir, \"{}\", \"{}\", &mut {});",
                    name.namespaces.join("::"),
                    name.as_data_type(),
                    table.table_path.display(),
                    table.schema.sheet,
                    handles_name,
                ));
            }

            let handles_code = handles.join("\n");
            let code = format!(
r#"    let mut {handles_name} = Vec::new();
{handles_code}

    join({handles_name}).await?;"#
            );
            level_handles.push(code);
        }

        Ok(format!(
r#"
use calamine::Reader;

pub async fn load_all(data_dir: &std::path::PathBuf) -> Result<(), {CRATE_PREFIX}::LoadError> {{
    type HandleType = tokio::task::JoinHandle<Result<(), {CRATE_PREFIX}::LoadError>>;

    fn add<T: {CRATE_PREFIX}::Loadable>(
        data_dir: &std::path::PathBuf,
        file_path: &str,
        sheet: &str,
        handles: &mut Vec<HandleType>,
    ) {{
        let file_path = data_dir.join(file_path);
        let sheet = sheet.to_owned();

        handles.push(tokio::task::spawn(async move {{
            let mut workbook: calamine::Ods<_> = calamine::open_workbook(file_path)?;
            let sheet = workbook
                .with_header_row(calamine::HeaderRow::Row({HEADER_ROWS}))
                .worksheet_range(&sheet)?;
            T::load(&sheet.rows().collect::<Vec<_>>())?;

            Ok(())
        }}));
    }}

    async fn join(handles: Vec<HandleType>) -> Result<(), {CRATE_PREFIX}::LoadError> {{
        for handle in handles {{
            match handle.await {{
                Ok(result) => result?,
                _  => panic!("Data loading task has failed!"),
            }}
        }}

        Ok(())
    }}

{level_handles_code}

    Ok(())
}}
"#,
            level_handles_code = level_handles.join("\n"),
        ))
    }
}
