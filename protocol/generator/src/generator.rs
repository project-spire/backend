use std::fs;

use glob::glob;
use serde::Deserialize;

use crate::*;

const TAB: &str= "    ";

#[derive(Debug)]
pub struct Generator {
    config: Config,
    categories: Vec<Category>,
    protocol_entries: Vec<ProtocolEntry>,
}

#[derive(Debug, Deserialize)]
struct Category {
    category: String,
    offset: u16,
    protocols: Vec<Protocol>,
}

fn protocol_handle_default() -> bool { true }
#[derive(Debug, Deserialize)]
struct Protocol {
    protocol: String,
    target: ProtocolTarget,
    #[serde(default = "protocol_handle_default")]
    handle: bool,
    #[serde(default)]
    handler: ProtocolHandler,
    
    #[serde(default, rename = "box")]
    __box: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ProtocolTarget {
    Client,
    Server,
    All,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
enum ProtocolHandler {
    #[default]
    Local,
    Global,
}

#[derive(Debug)]
struct ProtocolEntry {
    category: String,
    number: u16,
    protocol: Protocol,
}

impl Generator {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            categories: Vec::new(),
            protocol_entries: Vec::new(),
        }
    }

    pub fn collect(&mut self) -> Result<(), Error> {
        let category_files: Vec<PathBuf> = glob(self.config.schema_dir.join("game/*.json").to_str().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .collect();

        for category_file in &category_files {
            let category: Category = serde_json::from_str(&fs::read_to_string(&category_file)?)?;
            self.categories.push(category);
        }

        for mut category in self.categories.drain(..) {
            let mut number = category.offset;

            for protocol in category.protocols.drain(..) {
                self.protocol_entries.push(ProtocolEntry {
                    category: category.category.clone(),
                    number,
                    protocol,
                });
                number += 1;
            }
        }

        Ok(())
    }

    pub fn generate(&self) -> Result<(), Error> {
        if self.config.generate_impl {
            self.generate_impl()?;
        }

        if self.config.generate_handle {
            self.generate_handle()?;
        }

        Ok(())
    }

    fn generate_impl(&self) -> Result<(), Error> {
        let mut protocol_impls = Vec::new();
        let mut protocol_local_enums = Vec::new();
        let mut protocol_global_enums = Vec::new();
        let mut protocol_local_decodes = Vec::new();
        let mut protocol_global_decodes = Vec::new();
        let mut protocol_handler_enums = Vec::new();

        for entry in &self.protocol_entries {
            let protocol_full_name = format!("{}::{}", entry.category, entry.protocol.protocol);

            protocol_impls.push(format!(r#"
impl crate::game::Protocol for {protocol_full_name} {{
    fn protocol_id(&self) -> crate::game::ProtocolId {{ {} }}
}}
"#,
                entry.number,
            ));

            if !entry.protocol.handle {
                continue;
            }

            match &entry.protocol.target {
                ProtocolTarget::Server | ProtocolTarget::All => {},
                _ => continue,
            }
            
            let mut push = |
                protocol_enums: &mut Vec<String>,
                protocol_decodes: &mut Vec<String>,
                protocol_handler: &str,
            | {
                if entry.protocol.__box {
                    protocol_enums.push(format!(
                        "{TAB}{}(Box<{}>),",
                        entry.protocol.protocol,
                        protocol_full_name,
                    ));

                    protocol_decodes.push(format!(
                        "{TAB}{TAB}{} => {}(Box::new({protocol_full_name}::decode(data)?)),",
                        entry.number,
                        entry.protocol.protocol,
                    ));
                } else {
                    protocol_enums.push(format!(
                        "{TAB}{}({}),",
                        entry.protocol.protocol,
                        protocol_full_name,
                    ));

                    protocol_decodes.push(format!(
                        "{TAB}{TAB}{} => {}({protocol_full_name}::decode(data)?),",
                        entry.number,
                        entry.protocol.protocol,
                    ));
                }

                protocol_handler_enums.push(format!(
                    "{TAB}{TAB}{} => {},",
                    entry.number,
                    protocol_handler,
                ))
            };

            match entry.protocol.handler {
                ProtocolHandler::Local => push(
                    &mut protocol_local_enums,
                    &mut protocol_local_decodes,
                    "Local",
                ),
                ProtocolHandler::Global => push(
                    &mut protocol_global_enums,
                    &mut protocol_global_decodes,
                    "Global",
                ),
            }
        }

        let code = format!(r#"use prost::Message;

pub enum IngressLocalProtocol {{
{protocol_local_enums_code}
}}

pub enum IngressGlobalProtocol {{
{protocol_global_enums_code}
}}

pub fn protocol_handler(id: ProtocolId) -> Result<ProtocolHandler, Error> {{
    use ProtocolHandler::*;

    Ok(match id {{
{protocol_handler_enums_code}
        _ => return Err(Error::UnhandledProtocol(id)),
    }})
}}

pub fn decode_local(
    id: ProtocolId,
    data: bytes::Bytes,
) -> Result<IngressLocalProtocol, Error> {{
    use IngressLocalProtocol::*;

    Ok(match id {{
{protocol_local_decodes_code}
        _ => return Err(Error::ProtocolId(id)),
    }})
}}

pub fn decode_global(
    id: ProtocolId,
    data: bytes::Bytes,
) -> Result<IngressGlobalProtocol, Error> {{
    use IngressGlobalProtocol::*;

    Ok(match id {{
{protocol_global_decodes_code}
        _ => return Err(Error::ProtocolId(id)),
    }})
}}

{protocol_impls_code}
        "#,
            protocol_impls_code = protocol_impls.join("\n"),
            protocol_local_enums_code = protocol_local_enums.join("\n"),
            protocol_global_enums_code = protocol_global_enums.join("\n"),
            protocol_local_decodes_code = protocol_local_decodes.join("\n"),
            protocol_global_decodes_code = protocol_global_decodes.join("\n"),
            protocol_handler_enums_code = protocol_handler_enums.join("\n"),
        );

        let gen_file = PathBuf::from(&self.config.gen_dir).join("spire.protocol.game.impl.rs");
        fs::write(gen_file, &code)?;

        Ok(())
    }

    fn generate_handle(&self) -> Result<(), Error> {
        let mut protocol_local_handles = Vec::new();
        let mut protocol_global_handles = Vec::new();

        for entry in &self.protocol_entries {
            let protocol_full_name = format!("protocol::game::{}::{}", entry.category, entry.protocol.protocol);

            if !entry.protocol.handle {
                continue;
            }

            match &entry.protocol.target {
                ProtocolTarget::Server | ProtocolTarget::All => {},
                _ => continue,
            }

            match entry.protocol.handler {
                ProtocolHandler::Local => {
                    protocol_local_handles.push(format!(
                        "{TAB}{TAB}{}(p) => p.handle(entity, zone),",
                        entry.protocol.protocol,
                    ));
                }
                ProtocolHandler::Global => {
                    protocol_global_handles.push(format!(
                        "{TAB}{TAB}{}(p) => p.handle(entry),",
                        entry.protocol.protocol,
                    ));
                }
            }
        }

        let code = format!(r#"use protocol::game::{{IngressGlobalProtocol, IngressLocalProtocol}};

pub fn handle_local(
    protocol: IngressLocalProtocol,
    entity: bevy_ecs::entity::Entity,
    zone: &mut Zone,
) {{
    use IngressLocalProtocol::*;

    match protocol {{
{protocol_local_handles_code}
    }}
}}

pub fn handle_global(
    protocol: IngressGlobalProtocol,
    entry: Entry,
) {{
    use IngressGlobalProtocol::*;

    match protocol {{
{protocol_global_handles_code}
    }}
}}
"#,
            protocol_local_handles_code = protocol_local_handles.join("\n"),
            protocol_global_handles_code = protocol_global_handles.join("\n"),
        );

        let gen_file = PathBuf::from(&self.config.gen_dir).join("spire.protocol.game.handle.rs");
        fs::write(gen_file, &code)?;

        Ok(())
    }
}
