use std::collections::VecDeque;
use std::fs;

use crate::generator::*;
use crate::*;

const TUPLE_TYPES_MAX_COUNT: usize = 4;

#[derive(Debug)]
pub struct TableEntry {
    pub name: Name,
    pub schema: TableSchema,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum TableSchema {
    Concrete(ConcreteTableSchema),
    Abstract(AbstractTableSchema),
}

#[derive(Debug, Deserialize)]
pub struct ConcreteTableSchema {
    pub name: String,
    pub workbook: String,
    pub sheet: String,
    pub fields: Vec<Field>,
    pub extend: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AbstractTableSchema {
    pub name: String,
    pub fields: Vec<Field>,
    pub extend: Option<String>,
}

pub trait TableSchematic {
    fn name(&self) -> &str;
    fn fields(&self) -> &Vec<Field>;
    fn extend(&self) -> &Option<String>;
}

#[derive(Debug, Clone, Deserialize)]
pub struct Field {
    pub name: String,
    pub target: Target,
    #[serde(flatten)]
    pub kind: FieldKind,
    #[serde(default)]
    pub optional: Option<bool>,
    #[serde(default)]
    pub multi: Option<bool>,
    #[serde(default)]
    pub constraints: Option<Vec<Constraint>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum FieldKind {
    Scalar {
        #[serde(rename = "type")]
        scalar_type: ScalarAllType,
    },
    Enum {
        #[serde(rename = "type")]
        enum_type: String,
    },
    Link {
        #[serde(rename = "type")]
        link_type: String,
    },
    Tuple {
        types: Vec<FieldKind>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScalarAllType {
    Id,
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Float32,
    Float64,
    String,
    Datetime,
    Duration,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Constraint {
    Unique,
    Max(i32),
    Min(i32),
}

impl Generator {
    pub fn generate_table(&self, table: &TableEntry) -> Result<(), Error> {
        let table_file = self
            .full_gen_dir(&table.name.parent_namespace())
            .join(format!("{}.rs", table.name.as_entity()));
        self.log(&format!("Generating table `{}`", table_file.display()));

        let code = match &table.schema {
            TableSchema::Concrete(schema) => self.generate_concrete_table(&table.name, schema)?,
            TableSchema::Abstract(schema) => self.generate_abstract_table(&table.name, schema)?,
        };

        fs::write(table_file, code)?;

        Ok(())
    }

    fn generate_concrete_table(
        &self,
        name: &Name,
        schema: &ConcreteTableSchema,
    ) -> Result<String, Error> {
        let mut field_names = Vec::new();
        let mut field_parses = Vec::new();
        let mut field_definitions = Vec::new();
        let mut constraint_inits = Vec::new();
        let mut constraint_checks = Vec::new();
        let mut link_inits = Vec::new();

        let fields = self.get_table_all_fields(schema)?;
        for (index, field) in fields.iter().enumerate() {
            if !field.target.is_target() {
                continue;
            }

            let is_optional = field.optional.unwrap_or(false);
            let is_multi = field.multi.unwrap_or(false);

            if is_optional && is_multi {
                return Err(Error::InvalidAttribute(
                    "Optional and multi cannot be both true".to_string(),
                ));
            }

            let field_name = &field.name;
            field_names.push(field_name.clone());

            let mut is_unique = false;
            if let Some(constraints) = &field.constraints {
                for constraint in constraints {
                    match constraint {
                        Constraint::Unique => {
                            let set_name = format!("{field_name}_set");

                            constraint_inits.push(format!(
                                "{TAB}{TAB}let mut {set_name} = std::collections::HashSet::<{}>::new();",
                                field.kind.to_rust_type(),
                            ));
                            constraint_checks.push(format!(
                                r#"{TAB}{TAB}{TAB}if !{set_name}.insert(object.{field_name}.clone()) {{
                return Err(("{field_name}", ConstraintError::Unique {{
                    type_name: std::any::type_name::<{field_type}>(),
                    value: object.{field_name}.to_string(),
                }}));
            }}"#,
                                field_type = field.kind.to_rust_type(),
                            ));
                            is_unique = true;
                        }
                        Constraint::Max(value) => {
                            constraint_checks.push(format!(
                                r#"{TAB}{TAB}{TAB}if object.{field_name} > {value} {{
                return Err(("{field_name}", ConstraintError::Max {{
                    type_name: std::any::type_name::<{field_type}>(),
                    expected: {value}.to_string(),
                    actual: object.{field_name}.to_string(),
                }}));
            }}"#,
                                field_type = field.kind.to_rust_type(),
                            ));
                        }
                        Constraint::Min(value) => {
                            constraint_checks.push(format!(
                                r#"{TAB}{TAB}{TAB}if object.{field_name} < {value} {{
                return Err(("{field_name}", ConstraintError::Min {{
                    type_name: std::any::type_name::<{field_type}>(),
                    expected: {value}.to_string(),
                    actual: object.{field_name}.to_string(),
                }}));
            }}"#,
                                field_type = field.kind.to_rust_type(),
                            ));
                        }
                    }
                }
            }

            if is_multi && is_unique {
                return Err(Error::InvalidAttribute(
                    "Multi field cannot have unique constraint".to_string(),
                ));
            }

            field_definitions.push(format!("{TAB}pub {}: {},", field.name, {
                let base_type = field.kind.to_rust_type();
                let multi_type = if is_multi {
                    format!("Vec<{}>", base_type)
                } else {
                    base_type
                };
                if is_optional {
                    format!("Option<{}>", multi_type)
                } else {
                    multi_type
                }
            },));

            let field_parse = if is_optional {
                format!(
                    "{TAB}{TAB}let {field_name} = parse_optional(&row[{index}]).map_err(|e| (\"{field_name}\", e))?;"
                )
            } else if is_multi {
                format!(
                    "{TAB}{TAB}let {field_name} = parse_multi(&row[{index}]).map_err(|e| (\"{field_name}\", e))?;"
                )
            } else {
                format!(
                    "{TAB}{TAB}let {field_name} = parse(&row[{index}]).map_err(|e| (\"{field_name}\", e))?;"
                )
            };
            field_parses.push(field_parse);

            // Generate link initialization codes
            if !field.kind.has_link() {
                continue;
            }

            let link_init = if is_optional {
                let link_init = match &field.kind {
                    FieldKind::Link { .. } => {
                        format!(
                            "{TAB}{TAB}{TAB}{TAB}{TAB}{field_name}.init().map_err(|e| (*id, e))?;"
                        )
                    }
                    FieldKind::Tuple { types } => {
                        let mut inner_link_inits = Vec::new();
                        for (i, item) in types.iter().enumerate() {
                            match item {
                                FieldKind::Link { .. } => {}
                                _ => continue,
                            }

                            inner_link_inits.push(format!("{TAB}{TAB}{TAB}{TAB}{field_name}.{i}.init().map_err(|e| (*id, e))?;"));
                        }
                        inner_link_inits.join("\n")
                    }
                    _ => panic!("Invalid field type"),
                };

                format!(
                    r#"{TAB}{TAB}{TAB}{TAB}if let Some({field_name}) = row.{field_name}.as_mut() {{
{link_init}
                }}"#
                )
            } else if is_multi {
                let link_init = match &field.kind {
                    FieldKind::Link { .. } => {
                        format!("{TAB}{TAB}{TAB}{TAB}{TAB}x.init().map_err(|e| (*id, e))?;")
                    }
                    FieldKind::Tuple { types } => {
                        let mut inner_link_inits = Vec::new();
                        for (i, item) in types.iter().enumerate() {
                            match item {
                                FieldKind::Link { .. } => {}
                                _ => continue,
                            }

                            inner_link_inits.push(format!(
                                "{TAB}{TAB}{TAB}{TAB}{TAB}x.{i}.init().map_err(|e| (*id, e))?;"
                            ));
                        }
                        inner_link_inits.join("\n")
                    }
                    _ => panic!("Invalid field type"),
                };

                format!(
                    r#"{TAB}{TAB}{TAB}{TAB}for x in &mut row.{field_name} {{
{link_init}
                }}"#
                )
            } else {
                match &field.kind {
                    FieldKind::Link { .. } => {
                        format!(
                            "{TAB}{TAB}{TAB}{TAB}row.{field_name}.init().map_err(|e| (*id, e))?;"
                        )
                    }
                    FieldKind::Tuple { types } => {
                        let mut inner_link_inits = Vec::new();
                        for (i, item) in types.iter().enumerate() {
                            match item {
                                FieldKind::Link { .. } => {}
                                _ => continue,
                            }

                            inner_link_inits.push(format!(
                                "{TAB}{TAB}{TAB}row.{field_name}.{i}.init().map_err(|e| (*id, e))?;"
                            ));
                        }
                        inner_link_inits.join("\n")
                    }
                    _ => panic!("Invalid field type"),
                }
            };

            link_inits.push(link_init);
        }

        // Generate codes
        let data_cell_name = name.as_data_type_cell();
        let data_type_name = name.as_data_type(false);
        let table_type_name = name.as_type(false);

        let field_definitions_code = field_definitions.join("\n");
        let field_parses_code = field_parses.join("\n");
        let field_passes_code = field_names
            .iter()
            .map(|name| format!("{TAB}{TAB}{TAB}{name},"))
            .collect::<Vec<_>>()
            .join("\n");

        let parent_insert_code = if let Some(parent) = self.get_parent_table(&schema.extend) {
            let parent_full_name = parent.name.as_type(true);

            format!(
                r#"

        for (id, row) in unsafe {{ {data_cell_name}.assume_init_ref() }}.data.iter() {{
            {CRATE_PREFIX}::{parent_full_name}Data::insert(&id, {CRATE_PREFIX}::{parent_full_name}::{table_type_name}(row)).await?;
        }}"#
            )
        } else {
            String::new()
        };

        let link_inits_code = if link_inits.is_empty() {
            String::new()
        } else {
            format!(
                r#"
        (|| {{
            for (id, row) in &mut unsafe {{ {data_cell_name}.assume_init_mut() }}.data {{
{inits_code}
            }}

            Ok(())
        }})().map_err(|(id, error)| Error::Link {{
            workbook: "{workbook}",
            sheet: "{sheet}",
            id,
            error,
        }})?;
"#,
                inits_code = link_inits.join("\n\n"),
                workbook = schema.workbook,
                sheet = schema.sheet,
            )
        };

        let constraint_function_code = if constraint_checks.is_empty() {
            String::new()
        } else {
            format!(
                r#"
{constraint_inits_code}

        let mut check_constraint = |object: &{table_type_name}| -> Result<(), (&'static str, ConstraintError)> {{
{constraint_checks_code}

            Ok(())
        }};
"#,
                constraint_inits_code = constraint_inits.join("\n"),
                constraint_checks_code = constraint_checks.join("\n\n"),
            )
        };

        let constraint_call_code = if constraint_checks.is_empty() {
            String::new()
        } else {
            format!(
                r#"

            check_constraint(&object)
                .map_err(|(column, error)| Error::Constraint {{
                    workbook: "{workbook}",
                    sheet: "{sheet}",
                    row: index + 1,
                    column,
                    error,
                }})?;"#,
                workbook = schema.workbook,
                sheet = schema.sheet,
            )
        };

        Ok(format!(
            r#"{GENERATED_FILE_WARNING}
#![allow(static_mut_refs)]

use std::collections::HashMap;
use std::mem::MaybeUninit;
use tracing::info;
use {CRATE_PREFIX}::{{DataId, Link, error::*, parse::*}};

static mut {data_cell_name}: MaybeUninit<{data_type_name}> = MaybeUninit::uninit();

#[derive(Debug)]
pub struct {table_type_name} {{
{field_definitions_code}
}}

pub struct {data_type_name} {{
    data: HashMap<DataId, {table_type_name}>,
}}

impl {table_type_name} {{
    fn parse(row: &[calamine::Data]) -> Result<(DataId, Self), (&'static str, ParseError)> {{
        const FIELDS_COUNT: usize = {fields_count};

        if row.len() < FIELDS_COUNT {{
            return Err(("", ParseError::InvalidColumnCount {{ expected: FIELDS_COUNT, actual: row.len() }}));
        }}

{field_parses_code}

        Ok((id, Self {{
{field_passes_code}
        }}))
    }}
}}

impl {CRATE_PREFIX}::Linkable for {table_type_name} {{
    fn get(id: &DataId) -> Option<&'static Self> {{
        {data_type_name}::get(id)
    }}
}}

impl {data_type_name} {{
    pub fn get(id: &DataId) -> Option<&'static {table_type_name}> {{
        unsafe {{ {data_cell_name}.assume_init_ref() }}.data.get(&id)
    }}

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static {table_type_name})> {{
        unsafe {{ {data_cell_name}.assume_init_ref() }}.data.iter()
    }}
}}

impl {CRATE_PREFIX}::Loadable for {data_type_name} {{
    async fn load(rows: &[&[calamine::Data]]) -> Result<(), Error> {{
        let mut objects = HashMap::new();
        let mut index = {HEADER_ROWS};
{constraint_function_code}
        for row in rows {{
            let (id, object) = {table_type_name}::parse(row)
                .map_err(|(column, error)| Error::Parse {{
                    workbook: "{workbook}",
                    sheet: "{sheet}",
                    row: index + 1,
                    column,
                    error,
                }})?;

            if objects.contains_key(&id) {{
                return Err(Error::DuplicateId {{
                    type_name: std::any::type_name::<{table_type_name}>(),
                    id,
                    a: format!("{{:?}}", objects[&id]),
                    b: format!("{{:?}}", object),
                }});
            }}{constraint_call_code}

            objects.insert(id, object);

            index += 1;
        }}

        let data = Self {{ data: objects }};
        unsafe {{ {data_cell_name}.write(data); }}{parent_insert_code}

        info!("Loaded {{}} rows", rows.len());
        Ok(())
    }}

    fn init() -> Result<(), Error> {{{link_inits_code}
        Ok(())
    }}
}}
"#,
            workbook = schema.workbook,
            sheet = schema.sheet,
            fields_count = fields.len(),
        ))
    }

    fn generate_abstract_table(
        &self,
        name: &Name,
        schema: &AbstractTableSchema,
    ) -> Result<String, Error> {
        let data_cell_name = name.as_data_type_cell();
        let data_type_name = name.as_data_type(false);
        let table_type_name = name.as_type(false);

        let mut child_types = Vec::new();
        let mut child_id_matches = Vec::new();

        for index in &self.table_hierarchies[&self.table_indices[&name.as_type(true)]] {
            let child_table = &self.tables[*index];
            let child_name = child_table.name.as_type(false);
            let child_full_name = child_table.name.as_type(true);

            child_types.push(format!(
                "{TAB}{}(&'static {CRATE_PREFIX}::{}),",
                child_name, child_full_name,
            ));

            child_id_matches.push(format!(
                "{TAB}{TAB}{TAB}Self::{child_name}(x) => &x.id{},",
                match child_table.schema {
                    TableSchema::Concrete(_) => "",
                    TableSchema::Abstract(_) => "()",
                }
            ));
        }

        let parent_insert_code = if let Some(parent) = self.get_parent_table(&schema.extend) {
            let parent_full_name = parent.name.as_type(true);

            format!(
                r#"

        {CRATE_PREFIX}::{parent_full_name}Data::insert(id, {CRATE_PREFIX}::{parent_full_name}::{table_type_name}(&data[id])).await?;"#
            )
        } else {
            String::new()
        };
        let child_types_code = child_types.join("\n");
        let child_id_matches_code = child_id_matches.join("\n");

        Ok(format!(
            r#"{GENERATED_FILE_WARNING}
#![allow(static_mut_refs)]

use std::collections::HashMap;
use std::mem::MaybeUninit;
use tokio::sync::Mutex;
use {CRATE_PREFIX}::{{DataId, error::Error}};

static mut {data_cell_name}: MaybeUninit<{data_type_name}> = MaybeUninit::uninit();

#[derive(Debug)]
pub enum {table_type_name} {{
{child_types_code}
}}

#[derive(Debug)]
pub struct {data_type_name} {{
    data: HashMap<DataId, {table_type_name}>,
}}

impl {table_type_name} {{
    pub fn id(&self) -> &DataId {{
        match self {{
{child_id_matches_code}
        }}
    }}
}}

impl {CRATE_PREFIX}::Linkable for {table_type_name} {{
    fn get(id: &DataId) -> Option<&'static Self> {{
        {data_type_name}::get(id)
    }}
}}

impl {data_type_name} {{
    pub fn get(id: &DataId) -> Option<&'static {table_type_name}> {{
        let data = unsafe {{ &{data_cell_name}.assume_init_ref().data }};
        data.get(&id)
    }}

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static {table_type_name})> {{
        let data = unsafe {{ &{data_cell_name}.assume_init_ref().data }};
        data.iter()
    }}

    pub(crate) fn init() {{
        let data = Self {{ data: HashMap::new() }};
        unsafe {{ {data_cell_name}.write(data); }}
    }}

    pub(crate) async fn insert(id: &DataId, row: {table_type_name}) -> Result<(), Error> {{
        static LOCK: Mutex<()> = Mutex::const_new(());

        let data = unsafe {{ &mut {data_cell_name}.assume_init_mut().data }};
        let _ = LOCK.lock().await;

        if data.contains_key(id) {{
            return Err(Error::DuplicateId {{
                type_name: std::any::type_name::<{table_type_name}>(),
                id: *id,
                a: format!("{{:?}}", data[id]),
                b: format!("{{:?}}", row)
            }});
        }}
        data.insert(*id, row);{parent_insert_code}

        Ok(())
    }}
}}
"#
        ))
    }

    /// Get all fields of table including its parents
    pub fn get_table_all_fields(&self, schema: &dyn TableSchematic) -> Result<Vec<Field>, Error> {
        let mut fields = VecDeque::from(schema.fields().clone());
        let mut extend = schema.extend().clone();

        while let Some(parent) = extend.take() {
            let parent_index = self
                .table_indices
                .get(&parent)
                .ok_or_else(|| Error::Inheritance(schema.name().to_owned(), parent))?;
            let parent_table = &self.tables[*parent_index];
            let parent_schema = parent_table.schema.schematic();

            for field in parent_schema.fields().iter().rev() {
                fields.push_front(field.clone());
            }

            extend = parent_schema.extend().clone();
        }

        Ok(Vec::from(fields))
    }
}

impl TableSchema {
    pub fn schematic(&self) -> &dyn TableSchematic {
        match self {
            TableSchema::Concrete(c) => c,
            TableSchema::Abstract(a) => a,
        }
    }
}

impl TableSchematic for ConcreteTableSchema {
    fn name(&self) -> &str {
        &self.name
    }
    fn fields(&self) -> &Vec<Field> {
        &self.fields
    }
    fn extend(&self) -> &Option<String> {
        &self.extend
    }
}

impl TableSchematic for AbstractTableSchema {
    fn name(&self) -> &str {
        &self.name
    }
    fn fields(&self) -> &Vec<Field> {
        &self.fields
    }
    fn extend(&self) -> &Option<String> {
        &self.extend
    }
}

impl FieldKind {
    fn to_rust_type(&self) -> String {
        match &self {
            FieldKind::Scalar { scalar_type: t } => match t {
                ScalarAllType::Id => "DataId",
                ScalarAllType::Bool => "bool",
                ScalarAllType::Int8 => "i8",
                ScalarAllType::Int16 => "i16",
                ScalarAllType::Int32 => "i32",
                ScalarAllType::Int64 => "i64",
                ScalarAllType::Uint8 => "u8",
                ScalarAllType::Uint16 => "u16",
                ScalarAllType::Uint32 => "u32",
                ScalarAllType::Uint64 => "u64",
                ScalarAllType::Float32 => "f32",
                ScalarAllType::Float64 => "f64",
                ScalarAllType::String => "String",
                ScalarAllType::Datetime => "chrono::DateTime",
                ScalarAllType::Duration => "chrono::Duration",
            }
            .to_string(),
            FieldKind::Enum { enum_type } => {
                format!("{CRATE_PREFIX}::{enum_type}")
            }
            FieldKind::Link { link_type } => {
                format!("Link<{CRATE_PREFIX}::{link_type}>")
            }
            FieldKind::Tuple { types } => {
                let type_strings = to_tuple_type_strings(types);
                format!("({})", type_strings.join(", "))
            }
        }
    }

    fn has_link(&self) -> bool {
        match &self {
            FieldKind::Scalar { .. } => false,
            FieldKind::Enum { .. } => false,
            FieldKind::Link { .. } => true,
            FieldKind::Tuple { types } => {
                let mut has_link = false;
                for t in types {
                    match t {
                        FieldKind::Scalar { .. } => continue,
                        FieldKind::Enum { .. } => continue,
                        FieldKind::Link { .. } => {
                            has_link = true;
                            break;
                        }
                        FieldKind::Tuple { .. } => panic!("Nested tuples are not supported"),
                    }
                }

                has_link
            }
        }
    }
}

fn to_tuple_type_strings(fields: &[FieldKind]) -> Vec<String> {
    let mut type_strings = Vec::new();
    for t in fields {
        if let FieldKind::Tuple { .. } = t {
            panic!("Nested tuples are not supported");
        }

        type_strings.push(t.to_rust_type())
    }

    if type_strings.len() < 2 {
        panic!("Tuples must have at least two fields");
    }
    if type_strings.len() > TUPLE_TYPES_MAX_COUNT {
        panic!(
            "Tuples with more than {} types are not supported",
            TUPLE_TYPES_MAX_COUNT
        );
    }

    type_strings
}
