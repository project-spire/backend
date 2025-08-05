use std::collections::HashMap;
use std::fs;
use glob::glob;
use std::path::PathBuf;

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

        self.derive()?;

        Ok(())
    }

    fn derive(&self) -> Result<(), Box<dyn std::error::Error>> {
        let schema_dir = self.schema_dir.join("client");
        let schemas: Vec<PathBuf> = glob(&format!("{}/**/*.proto", schema_dir.display()))?
            .filter_map(Result::ok)
            .collect();

        let file_descriptor_set = &protox::compile(
            &schemas,
            [&self.schema_dir, &schema_dir]
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
            PathBuf::from(f.name()).file_name().unwrap() == "client.proto"
        }) {
            Some(fd) => fd,
            None => return Err("client.proto file not found!".into()),
        };

        let enum_descriptor = match file_descriptor.enum_type.iter().find(
            |e| e.name() == "ClientProtocol"
        ) {
            Some(e) => e,
            None => return Err("ClientProtocol enum type not found".into()),
        };

        let mut protocols = Vec::new();
        for enum_value in &enum_descriptor.value {
            let protocol_name = enum_value.name().to_owned();
            let number = enum_value.number() as u16;

            protocols.push((protocol_name, number));
        }

        let mut impls = Vec::new();
        for (protocol_name, number) in &protocols {
            let protocol_full_name = format!("{}::{}",
                packages.get(protocol_name).unwrap(),
                protocol_name,
            );

            let impl_code = format!(
r#"impl crate::protocol::Protocol for {protocol_full_name} {{
    fn protocol() -> u16 {{ {number} }}
}}
"#);
            impls.push(impl_code);
        }

        let code = format!(
r#"// Generated file

{impls_code}"#,
            impls_code = impls.join("\n"),
        );
        fs::write(&self.gen_dir.join("spire.protocol.impl.rs"), &code)?;

        Ok(())
    }
}
