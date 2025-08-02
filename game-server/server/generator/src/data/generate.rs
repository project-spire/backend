use std::collections::{HashMap, VecDeque};
use std::fs;
use serde::Deserialize;
use crate::data::{ConstDef, Entity, EnumDef, GenerateError, Generator, ModuleDef, TableDef};
use crate::{HEADER_ROWS, TAB};

const CRATE_PREFIX: &str = "crate::data";
const GENERATED_FILE_WARNING_CODE: &str = r#"// This is a generated file. DO NOT MODIFY."#;

type DependencyGraph = HashMap<String, Vec<String>>;

fn to_type_name(name: &str) -> String {
    use heck::ToUpperCamelCase;

    name.to_upper_camel_case()
}

fn to_entity_name(name: &str) -> String {
    use heck::ToSnakeCase;

    name.to_snake_case()
}

fn to_data_type_name(name: &str) -> String {
    use heck::ToUpperCamelCase;

    format!("{}Data", name.to_upper_camel_case())
}

fn to_data_cell_name(name: &str) -> String {
    use heck::ToSnakeCase;

    format!("{}_DATA", name.to_snake_case().to_uppercase())
}

impl Generator {
    pub fn generate(&self) -> Result<(), GenerateError> {
        println!("Generating...");

        fs::create_dir_all(&self.config.gen_dir)?;

        let mut dependencies = HashMap::new();
        for table in self.tables.values() {
            println!("Generating table: {:?}", table);
            self.generate_table(table, &mut dependencies)?
        }

        for module in &self.modules {
            println!("Generating module: {:?}", module);
            self.generate_module(module, &dependencies)?
        }

        for constant in self.constants.values() {
            println!("Generating const: {:?}", constant);
            self.generate_const(constant)?
        }

        for enumeration in self.enums.values() {
            println!("Generating enum: {:?}", enumeration);
            self.generate_enum(enumeration)?
        }

        Ok(())
    }

    fn generate_module(
        &self,
        module: &ModuleDef,
        dependencies: &DependencyGraph
    ) -> Result<(), GenerateError> {
        let module_dir = self.full_gen_dir(&module.namespaces);
        fs::create_dir_all(&module_dir)?;

        let module_base_dir = if module.namespaces.is_empty() {
            self.full_gen_dir(&Vec::new())
        } else {
            self.full_gen_dir(&module.namespaces[..module.namespaces.len() - 1])
        };

        let code = module.generate(dependencies)?;
        fs::write(
            module_base_dir.join(format!("{}.rs", module.name)),
            code,
        )?;

        Ok(())
    }

    fn generate_table(
        &self,
        table: &TableDef,
        dependencies: &mut DependencyGraph,
    ) -> Result<(), GenerateError> {
        let schema: TableSchema = serde_json::from_str(
            &fs::read_to_string(&table.schema_path)?
        )?;
        let table_dir = self.full_gen_dir(&table.namespaces);
        let code = schema.generate()?;

        fs::write(
            table_dir.join(format!("{}.rs", to_entity_name(&schema.name))),
            code,
        )?;

        // Add dependencies
        dependencies.insert(table.get_name_with_namespace(), Vec::new());
        for field in &schema.fields {
            let link_type = match &field.kind {
                FieldKind::Link { link_type } => link_type,
                _ => continue,
            };

            dependencies.get_mut(&schema.name).unwrap().push(link_type.clone());
        }

        Ok(())
    }

    fn generate_const(&self, constant: &ConstDef) -> Result<(), GenerateError> {
        let const_dir = self.full_gen_dir(&constant.namespaces);
        let code = "";

        fs::write(
            const_dir.join(format!("{}.rs", to_entity_name(&constant.name))),
            code,
        )?;

        Ok(())
    }

    fn generate_enum(&self, enumeration: &EnumDef) -> Result<(), GenerateError> {
        let schema: EnumSchema = serde_json::from_str(
            &fs::read_to_string(&enumeration.file_path)?
        )?;
        let enum_dir = self.full_gen_dir(&enumeration.namespaces);
        let code = schema.generate()?;

        fs::write(
            enum_dir.join(format!("{}.rs", to_entity_name(&enumeration.name))),
            code,
        )?;

        Ok(())
    }
}

impl ModuleDef {
    fn generate(
        &self,
        dependencies: &DependencyGraph,
    ) -> Result<String, GenerateError> {
        let is_base_module = self.name == "data" && self.namespaces.is_empty();
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        for entity in &self.entities {
            match entity {
                Entity::Module(name) => {
                    imports.push(format!("pub mod {};", name));
                },
                Entity::Table(name) => {
                    imports.push(format!("pub mod {};", name));
                    exports.push(format!(
                        "pub use {}::{{{type_name}, {data_type_name}, {data_cell_name}}};",
                        name,
                        type_name = to_type_name(&name),
                        data_type_name = to_data_type_name(&name),
                        data_cell_name = to_data_cell_name(&name),
                    ));
                },
                Entity::Const(name) => {
                    imports.push(format!("pub mod {};", name));
                },
                Entity::Enum(name) => {
                    let type_name = to_type_name(&name);

                    imports.push(format!("pub mod {};", name));
                    exports.push(format!("pub use {}::{};", name, type_name));
                },
            }
        }

        let imports_code = imports.join("\n");
        let exports_code = exports.join("\n");
        let base_module_code = if is_base_module {
            self.generate_base(dependencies)?
        } else {
            "".to_string()
        };

        Ok(format!(
            r#"{GENERATED_FILE_WARNING_CODE}
{imports_code}

{exports_code}

{base_module_code}
"#))
    }

    fn generate_base(
        &self,
        dependencies: &DependencyGraph,
    ) -> Result<String, GenerateError> {
        // Topological sort (Khan's algorithm)
        let mut in_degrees: HashMap<String, usize> = dependencies
            .iter()
            .map(|(node, _)| (node.clone(), 0))
            .collect();

        for deps in dependencies.values() {
            for dep in deps {
                if let Some(degree) = in_degrees.get_mut(dep) {
                    *degree += 1;
                }
            }
        }

        let mut queue: VecDeque<String> = in_degrees
            .iter()
            .filter(|&(_, &degree)| degree == 0)
            .map(|(node, _)| node.clone())
            .collect();
        let mut levels: Vec<Vec<String>> = Vec::new();
        let mut sorted_count = 0;

        while !queue.is_empty() {
            let level_size = queue.len();
            let mut current_level = Vec::with_capacity(level_size);

            for _ in 0..level_size {
                let u = queue.pop_front().unwrap();
                if let Some(nodes) = dependencies.get(&u) {
                    for v in nodes {
                        if let Some(degree) = in_degrees.get_mut(v) {
                            *degree -= 1;
                            if *degree == 0 {
                                queue.push_back(v.clone());
                            }
                        }
                    }
                }
                current_level.push(u.clone());
                sorted_count += 1;
            }
            levels.push(current_level);
        }

        // Check cycle
        if sorted_count != dependencies.len() {
            return Err(GenerateError::DependencyCycle)
        }

        let mut level_handles = Vec::new();
        for (i, level) in levels.iter().enumerate() {
            if level.is_empty() {
                continue;
            }

            let mut handles = Vec::new();
            for node in level {
                let (namespace, name) = {
                    let components = node.split("::").collect::<Vec<&str>>();
                    let namespace = components[..components.len() - 1].join("::");
                    let name = components[components.len() - 1];

                    (namespace, name)
                };
                let data_type_name = to_data_type_name(&name);

                handles.push(format!(
                    r#"{TAB}{TAB}handles.push(tokio::spawn(async {{
            let mut workbook: calamine::Ods<_> = calamine::open_workbook(data_dir.join())?;
            let range = reader.worksheet_range("todo")?;
            {CRATE_PREFIX}::{namespace}::{data_type_name}::load(range.rows().skip({HEADER_ROWS}))
        }}));"#,
                ));
            }

            let handles_code = handles.join("\n");
            let code = format!(r#"    let level{i}_handles = {{
        let mut handles = Vec::new();
{handles_code}
        handles
    }};

    for handle in level{i}_handles {{
        match handle.await {{
            Ok(result) => {{ result?; }},
            _  => panic!("Data loading task has failed!"),
        }}
    }}"#
            );
            level_handles.push(code);
        }

        let level_handles_code = level_handles.join("\n");

        Ok(format!(r#"pub async fn load_all(data_dir: &std::path::PathBuf) -> Result<(), {CRATE_PREFIX}::LoadError> {{
{level_handles_code}

    Ok(())
}}
"#
        ))
    }
}

impl TableDef {
    fn get_name_with_namespace(&self) -> String {
        format!(
            "{}::{}",
            self.namespaces.join("::"),
            self.name,
        )
    }
}

#[derive(Debug, Deserialize)]
struct TableSchema {
    pub name: String,
    pub kind: TableKind,
    pub sheet: String,
    pub fields: Vec<Field>,
}

impl TableSchema {
    fn generate(&self) -> Result<String, GenerateError> {
        match self.kind {
            TableKind::Concrete => self.generate_concrete(),
            TableKind::Abstract => todo!(),
        }
    }

    fn generate_concrete(&self) -> Result<String, GenerateError> {
        let table_type_name = &self.name;
        let sheet_name = &self.sheet;
        let data_type_name = to_data_type_name(&table_type_name);
        let data_cell_name = to_data_cell_name(&table_type_name);

        // Check fields
        let mut lifetime_code = String::new();
        let mut lifetime_parameter_code = String::new();

        let mut field_names = Vec::new();
        let mut field_parses = Vec::new();
        let mut field_definitions = Vec::new();

        for (index, field) in self.fields.iter().enumerate() {
            if let FieldKind::Link { .. } = &field.kind {
                lifetime_code = "<'a>".to_string();
                lifetime_parameter_code = "<'_>".to_string();
            }

            field_names.push(field.name.clone());
            field_definitions.push(format!("{TAB}pub {}: {},", field.name, field.kind.to_rust_type()));
            field_parses.push(format!(
                "{TAB}{TAB}let {field_name} = {parse_function}(&row[{index}])?;",
                field_name = field.name,
                parse_function = field.kind.to_parse_code(),
            ));
        }

        // Generate codes
        let field_definitions_code = field_definitions.join("\n");
        let field_parses_code = field_parses.join("\n");
        let field_passes_code = field_names
            .iter()
            .map(|name| {
                format!("{TAB}{TAB}{TAB}{name},")
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(format!(
            r#"{GENERATED_FILE_WARNING_CODE}
use tracing::info;
use {CRATE_PREFIX}::DataId;

pub static {data_cell_name}: tokio::sync::OnceCell<{data_type_name}> = tokio::sync::OnceCell::const_new();

#[derive(Debug)]
pub struct {table_type_name}{lifetime_code} {{
{field_definitions_code}
}}

impl {table_type_name}{lifetime_code} {{
    fn parse(row: &[calamine::Data]) -> Result<(DataId, Self), {CRATE_PREFIX}::LoadError> {{
{field_parses_code}

        Ok((id, Self {{
{field_passes_code}
        }}))
    }}
}}

impl{lifetime_code} crate::data::Linkable for {table_type_name}{lifetime_parameter_code} {{
    fn get(id: DataId) -> Option<&'static Self> {{
        {data_type_name}::get(id)
    }}
}}

pub struct {data_type_name}{lifetime_code} {{
    data: std::collections::HashMap<DataId, {table_type_name}{lifetime_code}>,
}}

impl{lifetime_code} {data_type_name}{lifetime_parameter_code} {{
    pub fn get(id: DataId) -> Option<&'static {table_type_name}> {{
        {data_cell_name}
            .get()
            .expect("{data_cell_name} is not initialized yet")
            .data
            .get(&id)
    }}

    pub fn load(rows: &[&[calamine::Data]]) -> Result<(), {CRATE_PREFIX}::LoadError> {{
        let mut objects = std::collections::HashMap::new();
        for row in rows {{
            let (id, object) = {table_type_name}::parse(row)?;

            if objects.contains_key(&id) {{
                return Err({CRATE_PREFIX}::LoadError::DuplicatedId {{
                    type_name: "{table_type_name}",
                    id,
                }});
            }}

            objects.insert(id, object);
        }}

        {data_cell_name}.set(Self {{ data: objects }});

        info!("Loaded {{}} rows", rows.len() - {HEADER_ROWS});
        Ok(())
    }}
}}
"#
        ))
    }
}

#[derive(Debug, Deserialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
enum TableKind {
    Concrete,
    Abstract
}

#[derive(Debug, Deserialize)]
struct Field {
    pub name: String,
    #[serde(flatten)] pub kind: FieldKind,
    #[serde(default)] pub desc: Option<String>,
    #[serde(default)] pub optional: Option<bool>,
    #[serde(default)] pub cardinality: Option<Cardinality>,
    #[serde(default)] pub constraints: Option<Vec<Constraint>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum FieldKind {
    Scalar { #[serde(rename = "type")] scalar_type: ScalarType },
    Enum { #[serde(rename = "type")] enum_type: String },
    Link { #[serde(rename = "type")] link_type: String }
}

impl FieldKind {
    fn to_rust_type(&self) -> String {
        match &self {
            FieldKind::Scalar { scalar_type: t } => match t {
                ScalarType::Id => "DataId".to_string(),
                ScalarType::Bool => "bool".to_string(),
                ScalarType::Int8 => "i8".to_string(),
                ScalarType::Int16 => "i16".to_string(),
                ScalarType::Int32 => "i32".to_string(),
                ScalarType::Int64 => "i64".to_string(),
                ScalarType::Uint8 => "u8".to_string(),
                ScalarType::Uint16 => "u16".to_string(),
                ScalarType::Uint32 => "u32".to_string(),
                ScalarType::Uint64 => "u64".to_string(),
                ScalarType::Float32 => "f32".to_string(),
                ScalarType::Float64 => "f64".to_string(),
                ScalarType::Str => "String".to_string(),
                ScalarType::Datetime => "chrono::DateTime".to_string(),
                ScalarType::Duration => "chrono::Duration".to_string(),
            },
            FieldKind::Enum { enum_type: t } => {
                format!("{CRATE_PREFIX}::{t}")
            },
            FieldKind::Link { link_type: t } => {
                format!("Link<'a, {CRATE_PREFIX}::{t}>")
            },
        }
    }

    fn to_parse_code(&self) -> String {
        match self {
            FieldKind::Scalar { scalar_type: t } => format!("{CRATE_PREFIX}::{}", match t {
                ScalarType::Id => "parse_id",
                ScalarType::Bool => "parse_bool",
                ScalarType::Int8 => "parse_i8",
                ScalarType::Int16 => "parse_i16",
                ScalarType::Int32 => "parse_i32",
                ScalarType::Int64 => "parse_i64",
                ScalarType::Uint8 => "parse_u8",
                ScalarType::Uint16 => "parse_u16",
                ScalarType::Uint32 => "parse_u32",
                ScalarType::Uint64 => "parse_u64",
                ScalarType::Float32 => "parse_f32",
                ScalarType::Float64 => "parse_f64",
                ScalarType::Str => "parse_str",
                ScalarType::Datetime => "parse_datetime",
                ScalarType::Duration => "parse_duration",
            }),
            FieldKind::Enum { enum_type: t } => format!("{CRATE_PREFIX}::{t}::parse"),
            FieldKind::Link { link_type: t } => format!("{CRATE_PREFIX}::parse_link::<{CRATE_PREFIX}::{t}>"),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Target {
    Client,
    Server,
    All,
}

impl Target {
    fn is_target(&self) -> bool {
        self == &Target::Server || self == &Target::All
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ScalarType {
    Id,
    Bool,
    Int8, Int16, Int32, Int64,
    Uint8, Uint16, Uint32, Uint64,
    Float32, Float64,
    Str,
    Datetime,
    Duration,
}

#[derive(Debug, Deserialize)]
enum Cardinality {
    Single,
    Multi
}

#[derive(Debug, Deserialize)]
enum Constraint {
    #[serde(rename = "exclusive")] Exclusive
}

#[derive(Debug, Deserialize)]
struct EnumSchema {
    name: String,
    base: EnumBase,
    enums: Vec<String>,
    target: Target,
    attributes: Vec<EnumAttribute>,
}

impl EnumSchema {
    pub fn generate(&self) -> Result<String, GenerateError> {
        let enum_type_name = &self.name;
        let base_type_name = self.base.to_rust_type();
        let mut enums = Vec::new();
        let mut enum_parses = Vec::new();
        let mut enum_intos = Vec::new();
        let mut enum_froms = Vec::new();
        let mut attributes = Vec::new();

        let mut index: u32 = 0;
        for e in &self.enums {
            enums.push(format!("{TAB}{e},"));
            enum_parses.push(format!("{TAB}{TAB}{TAB}\"{e}\" => Self::{e},"));
            enum_intos.push(format!("{TAB}{TAB}{TAB}Self::{e} => {index},"));
            enum_froms.push(format!("{TAB}{TAB}{TAB}{index} => Self::{e},"));

            index += 1;
        }

        for attribute in &self.attributes {
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
            r#"{GENERATED_FILE_WARNING_CODE}
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
"#
        ))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum EnumBase {
    Uint8,
    Uint16,
    Uint32,
}

impl EnumBase {
    fn to_rust_type(&self) -> String {
        match self {
            EnumBase::Uint8 => "u8",
            EnumBase::Uint16 => "u16",
            EnumBase::Uint32 => "u32",
        }.to_string()
    }
}

#[derive(Debug, Deserialize)]
struct EnumAttribute {
    target: Target,
    attribute: String,
}
