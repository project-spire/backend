use crate::generator::*;
use crate::*;
use super::*;

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

impl Generator {
    pub(super) fn generate_concrete_table(
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

            if field.optional && field.multi {
                return Err(Error::InvalidAttribute(
                    "Optional and multi cannot be both true".to_string(),
                ));
            }

            let field_name = &field.name;
            field_names.push(field_name.clone());

            let mut is_unique = false;
            for constraint in &field.constraints {
                match constraint {
                    Constraint::Unique => {
                        let set_name = format!("{field_name}_set");

                        constraint_inits.push(format!(
                            "{TAB}{TAB}let mut {set_name} = std::collections::HashSet::<{}>::new();",
                            field.kind.to_rust_type(),
                        ));
                        constraint_checks.push(format!(
                            r#"{TAB}{TAB}{TAB}if !{set_name}.insert(row.{field_name}.clone()) {{
            return Err(("{field_name}", ConstraintError::Unique {{
                type_name: std::any::type_name::<{field_type}>(),
                value: row.{field_name}.to_string(),
            }}));
        }}"#,
                            field_type = field.kind.to_rust_type(),
                        ));
                        is_unique = true;
                    }
                    Constraint::Max(value) => {
                        constraint_checks.push(format!(
                            r#"{TAB}{TAB}{TAB}if row.{field_name} > {value} {{
            return Err(("{field_name}", ConstraintError::Max {{
                type_name: std::any::type_name::<{field_type}>(),
                expected: {value}.to_string(),
                actual: row.{field_name}.to_string(),
            }}));
        }}"#,
                            field_type = field.kind.to_rust_type(),
                        ));
                    }
                    Constraint::Min(value) => {
                        constraint_checks.push(format!(
                            r#"{TAB}{TAB}{TAB}if row.{field_name} < {value} {{
            return Err(("{field_name}", ConstraintError::Min {{
                type_name: std::any::type_name::<{field_type}>(),
                expected: {value}.to_string(),
                actual: row.{field_name}.to_string(),
            }}));
        }}"#,
                            field_type = field.kind.to_rust_type(),
                        ));
                    }
                }
            }

            if field.multi && is_unique {
                return Err(Error::InvalidAttribute(
                    "Multi field cannot have unique constraint".to_string(),
                ));
            }

            field_definitions.push(format!("{TAB}pub {}: {},", field.name, {
                let base_type = field.kind.to_rust_type();
                let multi_type = if field.multi {
                    format!("Vec<{}>", base_type)
                } else {
                    base_type
                };
                if field.optional {
                    format!("Option<{}>", multi_type)
                } else {
                    multi_type
                }
            },));

            let field_parse = if field.optional {
                format!(
                    "{TAB}{TAB}let {field_name} = parse_optional(&row[{index}]).map_err(|e| (\"{field_name}\", e))?;"
                )
            } else if field.multi {
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

            let link_init = if field.optional {
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
            } else if field.multi {
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
        let table_cell_name = name.as_table_type_cell();
        let table_type_name = name.as_table_type(false);
        let row_type_name = name.as_type(false);

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

        for (id, row) in unsafe {{ {table_cell_name}.assume_init_ref() }}.rows.iter() {{
            {CRATE_PREFIX}::{parent_full_name}Table::insert(&id, {CRATE_PREFIX}::{parent_full_name}::{row_type_name}(row)).await?;
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
            for (id, row) in &mut unsafe {{ {table_cell_name}.assume_init_mut() }}.rows {{
{inits_code}
            }}

            Ok(())
        }})().map_err(|(id, error)| Error::Link {{
            workbook: WORKBOOK,
            sheet: SHEET,
            id,
            error,
        }})?;
"#,
                inits_code = link_inits.join("\n\n"),
            )
        };

        let constraint_function_code = if constraint_checks.is_empty() {
            String::new()
        } else {
            format!(
                r#"
{constraint_inits_code}

        let mut check_constraint = |row: &{row_type_name}| -> Result<(), (&'static str, ConstraintError)> {{
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
            r#"

            check_constraint(&parsed_row)
                .map_err(|(column, error)| Error::Constraint {
                    workbook: WORKBOOK,
                    sheet: SHEET,
                    row: index + 1,
                    column,
                    error,
                })?;"#.into()
        };

        Ok(format!(
            r#"{GENERATED_FILE_WARNING}
#![allow(static_mut_refs)]

use {CRATE_PREFIX}::{{DataId, Link, error::*, parse::*}};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use tracing::info;

const WORKBOOK: &str = "{workbook}";
const SHEET: &str = "{sheet}";

static mut {table_cell_name}: MaybeUninit<{table_type_name}> = MaybeUninit::uninit();

#[derive(Debug)]
pub struct {row_type_name} {{
{field_definitions_code}
}}

pub struct {table_type_name} {{
    rows: HashMap<DataId, {row_type_name}>,
}}

impl {row_type_name} {{
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

impl {CRATE_PREFIX}::Linkable for {row_type_name} {{
    fn get(id: &DataId) -> Option<&'static Self> {{
        {table_type_name}::get(id)
    }}
}}

impl {table_type_name} {{
    pub fn get(id: &DataId) -> Option<&'static {row_type_name}> {{
        unsafe {{ {table_cell_name}.assume_init_ref() }}.rows.get(&id)
    }}

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static {row_type_name})> {{
        unsafe {{ {table_cell_name}.assume_init_ref() }}.rows.iter()
    }}
}}

impl {CRATE_PREFIX}::Loadable for {table_type_name} {{
    async fn load(rows: &[&[calamine::Data]]) -> Result<(), Error> {{
        let mut parsed_rows = HashMap::new();
        let mut index = {HEADER_ROWS};
{constraint_function_code}
        for row in rows {{
            let (id, parsed_row) = {row_type_name}::parse(row)
                .map_err(|(column, error)| Error::Parse {{
                    workbook: WORKBOOK,
                    sheet: SHEET,
                    row: index + 1,
                    column,
                    error,
                }})?;

            if parsed_rows.contains_key(&id) {{
                return Err(Error::DuplicateId {{
                    type_name: std::any::type_name::<{row_type_name}>(),
                    id,
                    a: format!("{{:?}}", parsed_rows[&id]),
                    b: format!("{{:?}}", parsed_rows),
                }});
            }}{constraint_call_code}

            parsed_rows.insert(id, parsed_row);

            index += 1;
        }}

        let table = Self {{ rows: parsed_rows }};
        info!("Loaded {{}} rows", table.rows.len());

        unsafe {{ {table_cell_name}.write(table); }}{parent_insert_code}
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
}
