use std::fs;
use crate::data::*;
use crate::data::generate::*;

impl Generator {
    pub fn generate_enumeration(&self, enumeration: &EnumerationEntry) -> Result<(), GenerateError> {
        let enum_dir = self.full_gen_dir(&enumeration.name.namespaces);
        let code = enumeration.generate()?;

        fs::write(
            enum_dir.join(format!("{}.rs", enumeration.name.as_entity())),
            code,
        )?;

        Ok(())
    }
}

impl EnumerationEntry {
    pub fn generate(&self) -> Result<String, GenerateError> {
        let mut enums = Vec::new();
        let mut enum_parses = Vec::new();
        let mut enum_intos = Vec::new();
        let mut enum_froms = Vec::new();
        let mut attributes = Vec::new();

        let mut index: u32 = 0;
        for e in &self.schema.enums {
            enums.push(format!("{TAB}{e},"));
            enum_parses.push(format!("{TAB}{TAB}{TAB}\"{e}\" => Self::{e},"));
            enum_intos.push(format!("{TAB}{TAB}{TAB}Self::{e} => {index},"));
            enum_froms.push(format!("{TAB}{TAB}{TAB}{index} => Self::{e},"));

            index += 1;
        }

        for attribute in &self.schema.attributes {
            if !attribute.target.is_target() {
                continue;
            }

            attributes.push(attribute.attribute.clone());
        }

        let enums_code = enums.join("\n");
        let attributes_code = attributes.join("\n");
        let enum_parses_code = enum_parses.join("\n");
        let enum_intos_code = enum_intos.join("\n");
        let enum_froms_code = enum_froms.join("\n");

        Ok(format!(
            r#"{GENERATED_FILE_WARNING}
{attributes_code}
pub enum {enum_type_name} {{
{enums_code}
}}

impl {enum_type_name} {{
    pub fn parse(value: &calamine::Data) -> Result<Self, {CRATE_PREFIX}::LoadError> {{
        let enum_string = {CRATE_PREFIX}::parse_string(value)?;

        Ok(match enum_string.as_str() {{
{enum_parses_code}
            _ => return Err({CRATE_PREFIX}::LoadError::Parse(format!(
                "Invalid value \"{{enum_string}}\" for {enum_type_name}"
            ))),
        }})
    }}

    pub fn try_from(value: &{base_type_name}) -> Option<Self> {{
        Some(match value {{
{enum_froms_code}
            _ => return None,
        }})
    }}
}}

impl Into<{base_type_name}> for {enum_type_name} {{
    fn into(self) -> {base_type_name} {{
        match self {{
{enum_intos_code}
        }}
    }}
}}
"#,
            enum_type_name = self.name.as_type(false),
            base_type_name = self.schema.base.to_rust_type(),
        ))
    }
}

impl EnumerationBase {
    fn to_rust_type(&self) -> String {
        match self {
            EnumerationBase::Uint8 => "u8",
            EnumerationBase::Uint16 => "u16",
            EnumerationBase::Uint32 => "u32",
        }.to_string()
    }
}
