use glob::glob;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

const TAB: &str = "    ";

#[derive(Debug)]
pub struct Config {
    pub schema_dir: PathBuf,
    pub gen_dir: PathBuf,

    pub dry_run: bool,
}

impl Config {
    pub fn generate(self) -> Result<(), Box<dyn std::error::Error>> {
        // Generate schema codes
        let schemas: Vec<PathBuf> = glob(&format!("{}/**/*.proto", self.schema_dir.display()))?
            .filter_map(Result::ok)
            .collect();
        for schema in &schemas {
            println!("cargo:rerun-if-changed={}", schema.display());
        }
        prost_build::compile_protos(&schemas, &[&self.schema_dir])?;

        // Generate implementation code
        self.generate_impl_code()?;

        Ok(())
    }

    fn generate_impl_code(&self) -> Result<(), Box<dyn std::error::Error>> {
        let category_files: Vec<PathBuf> = glob(self.schema_dir.join("*.json").to_str().unwrap())?
            .filter_map(Result::ok)
            .collect();

        let mut categories = Vec::new();
        let mut protocols = Vec::new();

        for category_file in &category_files {
            let category: Category = serde_json::from_str(
                &fs::read_to_string(&category_file)?
            )?;
            categories.push(category);
        }

        for category in &categories {
            let mut number = category.offset;
            for protocol in &category.protocols {
                protocols.push((&category.category, protocol, number));
                number += 1;
            }
        }

        let mut enums = Vec::new();
        let mut decode_matches = Vec::new();
        let mut id_impls = Vec::new();

        for (category, protocol_name, number) in protocols {
            let protocol_full_name = format!("crate::protocol::{}::{}",
                                             category,
                                             protocol_name,
            );

            enums.push(format!(
                "{TAB}{protocol_name}({protocol_full_name}),",
            ));

            decode_matches.push(format!(
                "{TAB}{TAB}{TAB}{number} => Self::{protocol_name}({protocol_full_name}::decode(data)?),",
            ));

            id_impls.push(format!(
                r#"impl crate::protocol::Protocolic for {protocol_full_name} {{
    fn protocol_id(&self) -> u16 {{ {number} }}
}}
"#
            ));
        }

        let code = format!(
            r#"// Generated file

#[derive(Debug)]
pub enum Protocol {{
{enums_code}
}}

impl Protocol {{
    pub fn decode(id: u16, data: bytes::Bytes) -> Result<Self, crate::protocol::Error> {{
        Ok(match id {{
{decode_matches_code}
            _ => return Err(crate::protocol::Error::ProtocolId(id)),
        }})
    }}
}}

{id_impls_code}"#,
            enums_code = enums.join("\n"),
            decode_matches_code = decode_matches.join("\n"),
            id_impls_code = id_impls.join("\n"),
        );

        fs::write(&self.gen_dir.join("spire.protocol.impl.rs"), &code)?;

        Ok(())
    }
}

#[derive(Deserialize)]
struct Category {
    category: String,
    offset: u16,
    protocols: Vec<String>,
}
