use std::fs;
use crate::*;
use crate::generator::*;
use heck::ToSnakeCase;

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
    pub fn generate_enumeration(&self, enumeration: &EnumerationEntry) -> Result<(), Error> {
        let enumeration_file = self
            .full_gen_dir(&enumeration.name.parent_namespace())
            .join(format!("{}.rs", enumeration.name.as_entity()));
        self.log(&format!("Generating enumeration `{}`", enumeration_file.display()));

        let mut code = self.generate_code(&enumeration.schema)?;

        if enumeration.schema.protocol {
            code += &self.generate_protocol(&enumeration.name, &enumeration.schema)?;
        }

        if enumeration.schema.queryable {
            code += &self.generate_queryable(&enumeration.name, &enumeration.schema)?;
        }
        
        fs::write(enumeration_file, code)?;
        
        Ok(())
    }

    fn generate_code(&self, schema: &EnumerationSchema) -> Result<String, Error> {
        let mut enums = Vec::new();
        let mut enum_parses = Vec::new();
        let mut enum_intos = Vec::new();
        let mut enum_froms = Vec::new();
        let mut attributes = Vec::new();
        let mut imports = Vec::new();

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
            if !attribute.target.is_target() {
                continue;
            }

            attributes.push(attribute.attribute.clone());
        }
        if schema.queryable {
            imports.push("use std::io::Write;");
            imports.push("use diesel::{AsExpression, FromSqlRow};");
            imports.push("use diesel::deserialize::FromSql;");
            imports.push("use diesel::pg::{Pg, PgValue};");
            imports.push("use diesel::serialize::{IsNull, Output, ToSql};");

            attributes.push("#[derive(FromSqlRow, AsExpression)]".into());
            attributes.push(format!(
                "#[diesel(sql_type = crate::schema::sql_types::{})]",
                schema.name,
            ));
        }

        Ok(format!(
            r#"{GENERATED_FILE_WARNING}
use crate::error::ParseError;
use crate::parse::*;
{imports_code}

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
            imports_code = imports.join("\n"),
        ))
    }

    fn generate_protocol(&self, name: &Name, schema: &EnumerationSchema) -> Result<String, Error> {
        let protocol_file = self.config.protocol_gen_dir
            .join(format!("{}.gen.proto", name.name));
        self.log(&format!("Generating enumeration protocol `{}`", protocol_file.display()));

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
        fs::write(protocol_file, code)?;

        Ok(format!(
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
        ))
    }

    fn generate_queryable(&self, name: &Name, schema: &EnumerationSchema) -> Result<String, Error> {
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
        self.log(&format!("Generating enumeration sql `{}`", sql_file.display()));



        let sql = format!(r#"-- THIS IS A GENERATED FILE. DO NOT MODIFY.

create type {} as enum (
{enums_code}
);
"#,
                          name.name,
                          enums_code = sql_enums.join(",\n"),
        );
        fs::write(sql_file, sql)?;

        Ok(format!(r#"
impl ToSql<crate::schema::sql_types::{enum_type_name}, Pg> for {enum_type_name} {{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {{
        match *self {{
{to_sql_matches_code}
        }}
        Ok(IsNull::No)
    }}
}}

impl FromSql<crate::schema::sql_types::{enum_type_name}, Pg> for {enum_type_name} {{
    fn from_sql(bytes: PgValue<'_>) -> diesel::deserialize::Result<Self> {{
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

        ))
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
