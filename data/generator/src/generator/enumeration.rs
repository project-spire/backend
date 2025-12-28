use crate::*;
use crate::generator::*;
use heck::ToSnakeCase;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct EnumerationEntry {
    pub name: Name,
    pub schema: EnumerationSchema
}

#[derive(Debug, Deserialize)]
pub struct EnumerationSchema {
    name: String,
    base: EnumerationBase,
    enums: Vec<String>,
    target: Target,
    #[serde(default)]
    protocol: bool,
    #[serde(default)]
    queryable: bool,
    #[serde(default)]
    attributes: Vec<EnumerationAttribute>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnumerationBase {
    Uint8,
    Uint16,
    Uint32,
}

#[derive(Debug, Deserialize)]
pub struct EnumerationAttribute {
    target: Target,
    attribute: String,
}

impl Generator {
    pub fn collect_enumeration(
        &mut self,
        module: &mut ModuleEntry,
        file: &Path,
        name: Name,
    ) -> Result<(), Error> {
        println!("Collecting enumeration `{}`", file.display());

        self.register_type(&name.as_type(true))?;

        let schema: EnumerationSchema = serde_json::from_str(&fs::read_to_string(file)?)?;

        self.enumerations.push(EnumerationEntry { name, schema });
        module
            .entries
            .push(EntityEntry::EnumerationIndex(self.enumerations.len() - 1));

        Ok(())
    }

    pub fn generate_enumeration(
        &self,
        enumeration: &EnumerationEntry,
        writer: &mut dyn Write,
    ) -> Result<(), Error> {
        println!("Generating enumeration `{}`", enumeration.name.name);

        self.generate_source(&enumeration.schema, writer)?;

        if enumeration.schema.protocol {
            self.generate_protobuf(&enumeration.name, &enumeration.schema, writer)?;
        }

        if enumeration.schema.queryable {
            self.generate_queryable(&enumeration.name, &enumeration.schema, writer)?;
        }
        
        Ok(())
    }

    fn generate_source(
        &self,
        schema: &EnumerationSchema,
        writer: &mut dyn Write,
    ) -> Result<(), Error> {
        let mut enums = Vec::new();
        let mut enum_parses = Vec::new();
        let mut enum_intos = Vec::new();
        let mut enum_froms = Vec::new();
        let mut attributes = Vec::new();

        let mut index: u32 = 0;
        for e in &schema.enums {
            enums.push(format!("{TAB}{e},"));
            enum_parses.push(format!("{TAB}{TAB}{TAB}\"{e}\" => Self::{e},"));
            enum_intos.push(format!("{TAB}{TAB}{TAB}Self::{e} => {index},"));
            enum_froms.push(format!("{TAB}{TAB}{TAB}{index} => Self::{e},"));

            index += 1;
        }

        attributes.push("#[derive(Debug, Clone, Copy, PartialEq, Eq)]".into());
        for attribute in &schema.attributes {
            if !self.is_target(attribute.target) {
                continue;
            }

            attributes.push(attribute.attribute.clone());
        }
        if schema.queryable {
            attributes.push("#[derive(diesel::FromSqlRow, diesel::AsExpression)]".into());
            attributes.push(format!(
                "#[diesel(sql_type = crate::schema::sql_types::{})]",
                schema.name,
            ));
        }

        write!(writer,
r#"
{attributes_code}
pub enum {enum_type_name} {{
{enums_code}
}}

impl {enum_type_name} {{
    pub fn parse(value: &calamine::Data) -> Result<Self, ParseError> {{
        let enum_string = String::parse(value)?;

        Ok(match enum_string.as_str() {{
{enum_parses_code}
            _ => return Err(ParseError::InvalidValue {{
                type_name: std::any::type_name::<{enum_type_name}>(),
                value: enum_string,
            }}),
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
            enums_code = enums.join("\n"),
            enum_parses_code = enum_parses.join("\n"),
            enum_intos_code = enum_intos.join("\n"),
            enum_froms_code = enum_froms.join("\n"),
            enum_type_name = schema.name,
            base_type_name = schema.base.to_rust_type(),
            attributes_code = attributes.join("\n"),
        )?;
        Ok(())
    }

    fn generate_protobuf(
        &self,
        name: &Name,
        schema: &EnumerationSchema,
        writer: &mut dyn Write,
    ) -> Result<(), Error> {
        let protobuf_file = self.config.protobuf_gen_dir
            .join(format!("{}.gen.proto", name.name));
        println!("Generating enumeration protocol `{}`", protobuf_file.display());

        let mut enums = Vec::new();
        let mut enum_matches = Vec::new();

        let mut index: u32 = 0;
        for e in &schema.enums {
            enums.push(format!("{TAB}{e} = {index};"));
            enum_matches.push(format!("{TAB}{TAB}{TAB}Self::{e} => Target::{e},"));

            index += 1;
        }

        let code = format!(
r#"{GENERATED_FILE_WARNING}
syntax = "proto3";

package spire.protocol;

enum {enum_type_name} {{
{enums_code}
}}
"#,
            enum_type_name = name.as_type(false),
            enums_code = enums.join("\n"),
        );
        fs::write(protobuf_file, code)?;

        write!(writer,
r#"
impl Into<protocol::{enum_type_name}> for {enum_type_name} {{
    fn into(self) -> protocol::{enum_type_name} {{
        type Target = protocol::Race;

        match self {{
{enum_matches_code}
        }}
    }}
}}

impl Into<{enum_type_name}> for protocol::{enum_type_name} {{
    fn into(self) -> {enum_type_name} {{
        type Target = {enum_type_name};

        match self {{
{enum_matches_code}
        }}
    }}
}}

"#,
            enum_type_name = name.as_type(false),
            enum_matches_code = enum_matches.join("\n"),
        )?;
        Ok(())
    }

    fn generate_queryable(
        &self,
        name: &Name,
        schema: &EnumerationSchema,
        writer: &mut dyn Write,
    ) -> Result<(), Error> {
        let mut sql_enums = Vec::new();
        let mut to_sql_matches = Vec::new();
        let mut from_sql_matches = Vec::new();

        for e in &schema.enums {
            sql_enums.push(format!("{TAB}'{}'", e.to_snake_case()));

            to_sql_matches.push(format!(
                "{TAB}{TAB}{TAB}Self::{e} => out.write_all(b\"{}\")?,",
                e.to_snake_case(),
            ));

            from_sql_matches.push(format!(
                "{TAB}{TAB}{TAB}b\"{}\" => Self::{},",
                e.to_snake_case(),
                e,
            ));
        }


        let sql_file = self.config.sql_gen_dir
            .join(format!("{}.gen.sql", name.name));
        println!("Generating enumeration sql `{}`", sql_file.display());

        let sql = format!(r#"-- {GENERATED_FILE_WARNING}

create type {} as enum (
{enums_code}
);
"#,
                          name.name,
                          enums_code = sql_enums.join(",\n"),
        );
        fs::write(sql_file, sql)?;

        write!(writer,
r#"
impl diesel::serialize::ToSql<crate::schema::sql_types::{enum_type_name}, diesel::pg::Pg> for {enum_type_name} {{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>,
    ) -> diesel::serialize::Result {{
        match *self {{
{to_sql_matches_code}
        }}
        Ok(diesel::serialize::IsNull::No)
    }}
}}

impl diesel::deserialize::FromSql<crate::schema::sql_types::{enum_type_name}, diesel::pg::Pg> for {enum_type_name} {{
    fn from_sql(
        bytes: diesel::pg::PgValue<'_>,
    ) -> diesel::deserialize::Result<Self> {{
        Ok(match bytes.as_bytes() {{
{from_sql_matches_code}
             _ => return Err(format!(
                "Unrecognized {enum_type_name} enum variant {{}}",
                String::from_utf8_lossy(bytes.as_bytes()),
            ).into()),
        }})
    }}
}}
"#,
            enum_type_name = schema.name,
            to_sql_matches_code = to_sql_matches.join("\n"),
            from_sql_matches_code = from_sql_matches.join("\n"),

        )?;
        Ok(())
    }
}

impl EnumerationBase {
    fn to_rust_type(&self) -> &str {
        match self {
            EnumerationBase::Uint8 => "u8",
            EnumerationBase::Uint16 => "u16",
            EnumerationBase::Uint32 => "u32",
        }
    }
}
