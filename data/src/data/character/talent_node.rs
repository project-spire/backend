// This is a generated file. DO NOT MODIFY.
#![allow(static_mut_refs)]

use crate::{DataId, Link, error::*, parse::*};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use tracing::info;

const WORKBOOK: &str = "talent_node.ods";
const SHEET: &str = "TalentNode";

static mut TALENT_NODE_TABLE: MaybeUninit<TalentNodeTable> = MaybeUninit::uninit();

#[derive(Debug)]
pub struct TalentNode {
    pub id: DataId,
}

pub struct TalentNodeTable {
    rows: HashMap<DataId, TalentNode>,
}

impl TalentNode {
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

impl crate::Linkable for TalentNode {
    fn get(id: &DataId) -> Option<&'static Self> {
        TalentNodeTable::get(id)
    }
}

impl TalentNodeTable {
    pub fn get(id: &DataId) -> Option<&'static TalentNode> {
        unsafe { TALENT_NODE_TABLE.assume_init_ref() }.rows.get(&id)
    }

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static TalentNode)> {
        unsafe { TALENT_NODE_TABLE.assume_init_ref() }.rows.iter()
    }
}

impl crate::Loadable for TalentNodeTable {
    async fn load(rows: &[&[calamine::Data]]) -> Result<(), Error> {
        let mut parsed_rows = HashMap::new();
        let mut index = 2;

        for row in rows {
            let (id, parsed_row) = TalentNode::parse(row)
                .map_err(|(column, error)| Error::Parse {
                    workbook: WORKBOOK,
                    sheet: SHEET,
                    row: index + 1,
                    column,
                    error,
                })?;

            if parsed_rows.contains_key(&id) {
                return Err(Error::DuplicateId {
                    type_name: std::any::type_name::<TalentNode>(),
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

        unsafe { TALENT_NODE_TABLE.write(table); }
        Ok(())
    }

    fn init() -> Result<(), Error> {
        Ok(())
    }
}
