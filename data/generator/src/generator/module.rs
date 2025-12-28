use std::fs;
use glob::glob;
use crate::*;
use crate::generator::*;
use crate::generator::table::TableSchema;

#[derive(Debug)]
pub struct ModuleEntry {
    pub name: Name,
    pub entries: Vec<EntityEntry>,
}

#[derive(Debug)]
pub enum EntityEntry {
    ModuleIndex(usize),
    TableIndex(usize),
    EnumerationIndex(usize),
    ConstantIndex(usize),
}

impl ModuleEntry {
    pub fn new(name: Name) -> Self {
        Self {
            name,
            entries: Vec::new(),
        }
    }
}

impl Generator {
    pub fn collect_module(
        &mut self,
        module: &mut ModuleEntry,
    ) -> Result<Vec<ModuleEntry>, Error> {
        let module_dir = module.name.as_full_dir(&self.config.schema_dir);
        println!("Collecting module `{}`", &module_dir.display());

        // Collect entities
        let mut entity_files: Vec<PathBuf> = glob(module_dir.join("*.json").to_str().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        entity_files.sort();

        println!("Found entity files: {:?}", &entity_files);

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

            println!("Found child directory `{}`", dir_name);
            child_modules.push(ModuleEntry::new(module_name));
        }
        child_modules.sort_by(|a, b| a.name.name.cmp(&b.name.name));

        Ok(child_modules)
    }

    pub fn generate_initialize(
        &self,
        writer: &mut dyn Write,
    ) -> Result<(), Error> {
        let mut abstract_table_inits = Vec::new();
        for table in &self.tables {
            match &table.schema {
                TableSchema::Concrete(_) => continue,
                TableSchema::Abstract(_) => {},
            };

            abstract_table_inits.push(
                format!(
                    "{TAB}crate::{}Table::init();",
                    table.name.as_type(true),
                )
            );
        }

        let mut concrete_table_loads = Vec::new();
        for table in &self.tables {
            let schema = match &table.schema {
                TableSchema::Concrete(schema) => schema,
                TableSchema::Abstract(_) => continue,
            };

            let file = {
                let mut parent_dir = table.name.parent_namespace().join("/");
                if !parent_dir.is_empty() {
                    parent_dir.push_str("/");
                }
                parent_dir + &schema.workbook
            };

            concrete_table_loads.push(
                format!(
                    "{TAB}load::<crate::{}>({}, \"{}\", &mut tasks);",
                    table.name.as_table_type(true),
                    format!("data_dir.join(\"{}\")", file),
                    schema.sheet,
                )
            );
        }

        let mut concrete_table_inits = Vec::new();
        for table in &self.tables {
            match &table.schema {
                TableSchema::Concrete(_) => {},
                TableSchema::Abstract(_) => continue,
            };

            concrete_table_inits.push(
                format!(
                    "{TAB}init::<crate::{}>(&mut tasks);",
                    table.name.as_table_type(true),
                )
            );
        }

        write!(writer,
r#"
pub async fn init(data_dir: &std::path::PathBuf) -> Result<(), Error> {{
    init_abstract_tables();
    load_concrete_tables(data_dir).await?;
    init_concrete_tables().await?;

    Ok(())
}}

fn init_abstract_tables() {{
{abstract_table_inits_code}
}}

async fn load_concrete_tables(data_dir: &std::path::PathBuf) -> Result<(), Error> {{
    fn load<T: crate::Loadable>(
        file: std::path::PathBuf,
        sheet: &str,
        tasks: &mut Vec<tokio::task::JoinHandle<Result<(), Error>>>,
    ) {{
        use calamine::Reader;
    
        let sheet = sheet.to_owned();

        tasks.push(tokio::task::spawn(async move {{
            let mut workbook: calamine::Ods<_> = calamine::open_workbook(&file)
                .map_err(|error| Error::Workbook {{
                    workbook: file.display().to_string(),
                    error,
                }})?;
            let sheet = workbook
                .with_header_row(calamine::HeaderRow::Row({header_rows}))
                .worksheet_range(&sheet)
                .map_err(|error| Error::Sheet {{
                    workbook: file.display().to_string(),
                    sheet,
                    error,
                }})?;
            T::load(&sheet.rows().collect::<Vec<_>>()).await?;

            Ok(())
        }}));
    }}

    let mut tasks = Vec::new();

{concrete_table_loads_code}

    for task in tasks {{
        match task.await {{
            Ok(result) => result?,
            _  => panic!("Data loading task has failed!"),
        }}
    }}
    Ok(())
}}

async fn init_concrete_tables() -> Result<(), Error> {{
    fn init<T: crate::Loadable>(
        tasks: &mut Vec<tokio::task::JoinHandle<Result<(), Error>>>,
    ) {{
        tasks.push(tokio::task::spawn(async move {{
            T::init()?;
            Ok(())
        }}));
    }}

    let mut tasks = Vec::new();

{concrete_table_inits_code}

    for task in tasks {{
        match task.await {{
            Ok(result) => result?,
            _  => panic!("Data initializing task has failed!"),
        }}
    }}
    Ok(())
}}
"#,
            header_rows = self.config.header_rows,
            abstract_table_inits_code = abstract_table_inits.join("\n"),
            concrete_table_loads_code = concrete_table_loads.join("\n"),
            concrete_table_inits_code = concrete_table_inits.join("\n"),
        )?;
        Ok(())
    }
}
