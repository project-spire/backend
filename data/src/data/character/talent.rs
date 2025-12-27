// This is a generated file. DO NOT MODIFY.
#![allow(static_mut_refs)]

use crate::{DataId, Link, error::*, parse::*};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use tracing::info;

const WORKBOOK: &str = "talent.ods";
const SHEET: &str = "Talent";

static mut TALENT_TABLE: MaybeUninit<TalentTable> = MaybeUninit::uninit();

#[derive(Debug)]
pub struct Talent {
    pub id: DataId,
}

pub struct TalentTable {
    rows: HashMap<DataId, Talent>,
}

impl Talent {
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

impl crate::Linkable for Talent {
    fn get(id: &DataId) -> Option<&'static Self> {
        TalentTable::get(id)
    }
}

impl TalentTable {
    pub fn get(id: &DataId) -> Option<&'static Talent> {
        unsafe { TALENT_TABLE.assume_init_ref() }.rows.get(&id)
    }

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static Talent)> {
        unsafe { TALENT_TABLE.assume_init_ref() }.rows.iter()
    }
}

impl crate::Loadable for TalentTable {
    async fn load(rows: &[&[calamine::Data]]) -> Result<(), Error> {
        let mut parsed_rows = HashMap::new();
        let mut index = 2;

        for row in rows {
            let (id, parsed_row) = Talent::parse(row)
                .map_err(|(column, error)| Error::Parse {
                    workbook: WORKBOOK,
                    sheet: SHEET,
                    row: index + 1,
                    column,
                    error,
                })?;

            if parsed_rows.contains_key(&id) {
                return Err(Error::DuplicateId {
                    type_name: std::any::type_name::<Talent>(),
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

        unsafe { TALENT_TABLE.write(table); }
        Ok(())
    }

    fn init() -> Result<(), Error> {
        Ok(())
    }
}
