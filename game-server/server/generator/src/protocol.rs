use std::collections::HashMap;
use std::fs;
use glob::glob;
use std::path::PathBuf;

const TAB: &str = "    ";
const PROTOCOL_ID_FILE_NAME: &str = "protocol_id.proto";
const PROTOCOL_ID_ENUM_NAME: &str = "ProtocolId";

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
        let schemas: Vec<PathBuf> = glob(&self.schema_dir.join("**/*.proto").to_str().unwrap())?
            .filter_map(Result::ok)
            .collect();

        let file_descriptor_set = &protox::compile(
            &schemas,
            [&self.schema_dir]
        )?;

        let mut packages = HashMap::new();
        for file_descriptor in &file_descriptor_set.file {
            for message_descriptor in &file_descriptor.message_type {
                packages.insert(
                    message_descriptor.name().to_owned(),
                    file_descriptor.package().replace(".", "::").replace("spire", "crate"),
                );
            }
        }

        let file_descriptor = match file_descriptor_set.file.iter().find(|f| {
            PathBuf::from(f.name()).file_name().unwrap() == PROTOCOL_ID_FILE_NAME
        }) {
            Some(fd) => fd,
            None => return Err(format!("{PROTOCOL_ID_FILE_NAME} file not found!").into()),
        };

        let enum_descriptor = match file_descriptor.enum_type.iter().find(
            |e| e.name() == PROTOCOL_ID_ENUM_NAME
        ) {
            Some(e) => e,
            None => return Err(format!("{PROTOCOL_ID_ENUM_NAME} enum type not found").into()),
        };

        let mut protocols = Vec::new();
        for enum_value in &enum_descriptor.value {
            let protocol_name = enum_value.name().to_owned();
            let number = enum_value.number() as u16;

            protocols.push((protocol_name, number));
        }

        let mut enums = Vec::new();
        let mut decode_matches = Vec::new();
        let mut id_impls = Vec::new();

        for (protocol_name, number) in &protocols {
            let protocol_full_name = format!("{}::{}",
                packages.get(protocol_name).unwrap(),
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
