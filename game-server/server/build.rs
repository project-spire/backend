use heck::ToSnakeCase;
use serde::Deserialize;
use std::{env, fs};
use std::collections::HashMap;
use std::path::PathBuf;
use game_data::generator::{*, table::*};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(env::var("OUT_DIR")?).join("gen");
    fs::create_dir_all(&out_dir)?;

    // Copy Settings file
    // println!("cargo:rerun-if-changed=settings.ron");
    // fs::copy("../settings.ron", out_dir.join("../settings.ron"))?;

    generate_game_data_code(&out_dir)?;

    Ok(())
}

fn generate_game_data_code(out_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let data_root = PathBuf::from("../game-data/data");
    println!("cargo:rerun-if-changed={}", data_root.join("data.json").display());

    let data_entries: Vec<DataEntry> = serde_json::from_str(
        &fs::read_to_string(&data_root.join("data.json"))?
    )?;

    // Collect table entries
    let mut table_entries = Vec::new();
    for entry in data_entries {
        let file_path = data_root.join(entry.file);
        let schema_path = data_root.join(entry.schema);

        println!("cargo:rerun-if-changed={}", file_path.display());
        println!("cargo:rerun-if-changed={}", schema_path.display());

        let schema_str = fs::read_to_string(&schema_path)?;
        println!("{}", &schema_str);
        let schema: Schema = serde_json::from_str(&schema_str)?;

        if schema.tables.is_empty() {
            panic!("Tables of schema {} is empty!", schema_path.display());
        }

        table_entries.push(TableEntry { file_path, schema });
    }

    // Register types
    let mut table_types = HashMap::new();
    for entry in &table_entries {
        register_table_types(&entry.schema.tables, &mut table_types)?;
    }

    // Generate codes
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    for entry in &table_entries {
        for table in &entry.schema.tables {
            let mod_name = table.name.to_snake_case();
            let path = out_dir.join(format!("{}.rs", &mod_name));
            println!("{}", path.display());
            let code = generate_table_code(&table, &table_types)?;
            fs::write(path, code)?;
            
            imports.push(format!("pub mod {mod_name};"));
            exports.push(format!(
                "pub use {}::{{{}, {}}};",
                mod_name,
                table.name,
                format!("{}Data", table.name),
            ));
        }
    }
    
    // Generate data module
    let imports_code = imports.join("\n");
    let exports_code = exports.join("\n");
    let code = format!(
r#"// Generated file
{imports_code}

{exports_code}
"#
    );
    let path = out_dir.join("data.rs");
    fs::write(path, code)?;

    Ok(())
}

struct TableEntry {
    file_path: PathBuf,
    schema: Schema,
}

#[derive(Deserialize)]
struct DataEntry {
    file: String,
    schema: String,
}

#[derive(Deserialize)]
struct Schema {
    tables: Vec<Table>
}
