// This is a generated file. DO NOT MODIFY.
#![allow(static_mut_refs)]

use crate::{DataId, Link, error::*, parse::*};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use tracing::info;

const WORKBOOK: &str = "random_box.ods";
const SHEET: &str = "RandomBox";

static mut RANDOM_BOX_TABLE: MaybeUninit<RandomBoxTable> = MaybeUninit::uninit();

#[derive(Debug)]
pub struct RandomBox {
    pub id: DataId,
    pub name: String,
    pub items: Vec<(Link<crate::item::Item>, u16)>,
}

pub struct RandomBoxTable {
    rows: HashMap<DataId, RandomBox>,
}

impl RandomBox {
    fn parse(row: &[calamine::Data]) -> Result<(DataId, Self), (&'static str, ParseError)> {
        const FIELDS_COUNT: usize = 3;

        if row.len() < FIELDS_COUNT {
            return Err(("", ParseError::InvalidColumnCount { expected: FIELDS_COUNT, actual: row.len() }));
        }

        let id = parse(&row[0]).map_err(|e| ("id", e))?;
        let name = parse(&row[1]).map_err(|e| ("name", e))?;
        let items = parse_multi(&row[2]).map_err(|e| ("items", e))?;

        Ok((id, Self {
            id,
            name,
            items,
        }))
    }
}

impl crate::Linkable for RandomBox {
    fn get(id: &DataId) -> Option<&'static Self> {
        RandomBoxTable::get(id)
    }
}

impl RandomBoxTable {
    pub fn get(id: &DataId) -> Option<&'static RandomBox> {
        unsafe { RANDOM_BOX_TABLE.assume_init_ref() }.rows.get(&id)
    }

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static RandomBox)> {
        unsafe { RANDOM_BOX_TABLE.assume_init_ref() }.rows.iter()
    }
}

impl crate::Loadable for RandomBoxTable {
    async fn load(rows: &[&[calamine::Data]]) -> Result<(), Error> {
        let mut parsed_rows = HashMap::new();
        let mut index = 2;

        for row in rows {
            let (id, parsed_row) = RandomBox::parse(row)
                .map_err(|(column, error)| Error::Parse {
                    workbook: WORKBOOK,
                    sheet: SHEET,
                    row: index + 1,
                    column,
                    error,
                })?;

            if parsed_rows.contains_key(&id) {
                return Err(Error::DuplicateId {
                    type_name: std::any::type_name::<RandomBox>(),
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

        unsafe { RANDOM_BOX_TABLE.write(table); }

        for (id, row) in unsafe { RANDOM_BOX_TABLE.assume_init_ref() }.rows.iter() {
            crate::item::ItemTable::insert(&id, crate::item::Item::RandomBox(row)).await?;
        }
        Ok(())
    }

    fn init() -> Result<(), Error> {
        (|| {
            for (id, row) in &mut unsafe { RANDOM_BOX_TABLE.assume_init_mut() }.rows {
                for x in &mut row.items {
                    x.0.init().map_err(|e| (*id, e))?;
                }
            }

            Ok(())
        })().map_err(|(id, error)| Error::Link {
            workbook: WORKBOOK,
            sheet: SHEET,
            id,
            error,
        })?;

        Ok(())
    }
}
