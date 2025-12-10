// This is a generated file. DO NOT MODIFY.
#![allow(static_mut_refs)]

use crate::{DataId, Link, error::*, parse::*};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use tracing::info;

const WORKBOOK: &str = "path_node.ods";
const SHEET: &str = "PathNode";

static mut PATH_NODE_TABLE: MaybeUninit<PathNodeTable> = MaybeUninit::uninit();

#[derive(Debug)]
pub struct PathNode {
    pub id: DataId,
}

pub struct PathNodeTable {
    rows: HashMap<DataId, PathNode>,
}

impl PathNode {
    fn parse(row: &[calamine::Data]) -> Result<(DataId, Self), (&'static str, ParseError)> {
        const FIELDS_COUNT: usize = 2;

        if row.len() < FIELDS_COUNT {
            return Err(("", ParseError::InvalidColumnCount { expected: FIELDS_COUNT, actual: row.len() }));
        }

        let id = parse(&row[0]).map_err(|e| ("id", e))?;

        Ok((id, Self {
            id,
        }))
    }
}

impl crate::Linkable for PathNode {
    fn get(id: &DataId) -> Option<&'static Self> {
        PathNodeTable::get(id)
    }
}

impl PathNodeTable {
    pub fn get(id: &DataId) -> Option<&'static PathNode> {
        unsafe { PATH_NODE_TABLE.assume_init_ref() }.rows.get(&id)
    }

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static PathNode)> {
        unsafe { PATH_NODE_TABLE.assume_init_ref() }.rows.iter()
    }
}

impl crate::Loadable for PathNodeTable {
    async fn load(rows: &[&[calamine::Data]]) -> Result<(), Error> {
        let mut parsed_rows = HashMap::new();
        let mut index = 2;

        for row in rows {
            let (id, parsed_row) = PathNode::parse(row)
                .map_err(|(column, error)| Error::Parse {
                    workbook: WORKBOOK,
                    sheet: SHEET,
                    row: index + 1,
                    column,
                    error,
                })?;

            if parsed_rows.contains_key(&id) {
                return Err(Error::DuplicateId {
                    type_name: std::any::type_name::<PathNode>(),
                    id,
                    a: format!("{:?}", parsed_rows[&id]),
                    b: format!("{:?}", parsed_rows),
                });
            }

            parsed_rows.insert(id, parsed_row);

            index += 1;
        }

        let table = Self { rows: parsed_rows };
        info!("Loaded {} rows", table.rows.len());

        unsafe { PATH_NODE_TABLE.write(table); }
        Ok(())
    }

    fn init() -> Result<(), Error> {
        Ok(())
    }
}
