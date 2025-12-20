// This is a generated file. DO NOT MODIFY.
#![allow(static_mut_refs)]

use crate::{DataId, Link, error::*, parse::*};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use tracing::info;

const WORKBOOK: &str = "skill.ods";
const SHEET: &str = "GenericSkill";

static mut GENERIC_SKILL_TABLE: MaybeUninit<GenericSkillTable> = MaybeUninit::uninit();

#[derive(Debug)]
pub struct GenericSkill {
    pub id: DataId,
    pub effects: Vec<Link<crate::skill::SkillEffect>>,
    pub costs: Vec<crate::skill::SkillCost>,
}

pub struct GenericSkillTable {
    rows: HashMap<DataId, GenericSkill>,
}

impl GenericSkill {
    fn parse(row: &[calamine::Data]) -> Result<(DataId, Self), (&'static str, ParseError)> {
        const FIELDS_COUNT: usize = 4;

        if row.len() < FIELDS_COUNT {
            return Err(("", ParseError::InvalidColumnCount { expected: FIELDS_COUNT, actual: row.len() }));
        }

        let id = parse(&row[0]).map_err(|e| ("id", e))?;
        let effects = parse_multi(&row[2]).map_err(|e| ("effects", e))?;
        let costs = parse_multi(&row[3]).map_err(|e| ("costs", e))?;

        Ok((id, Self {
            id,
            effects,
            costs,
        }))
    }
}

impl crate::Linkable for GenericSkill {
    fn get(id: &DataId) -> Option<&'static Self> {
        GenericSkillTable::get(id)
    }
}

impl GenericSkillTable {
    pub fn get(id: &DataId) -> Option<&'static GenericSkill> {
        unsafe { GENERIC_SKILL_TABLE.assume_init_ref() }.rows.get(&id)
    }

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static GenericSkill)> {
        unsafe { GENERIC_SKILL_TABLE.assume_init_ref() }.rows.iter()
    }
}

impl crate::Loadable for GenericSkillTable {
    async fn load(rows: &[&[calamine::Data]]) -> Result<(), Error> {
        let mut parsed_rows = HashMap::new();
        let mut index = 2;

        for row in rows {
            let (id, parsed_row) = GenericSkill::parse(row)
                .map_err(|(column, error)| Error::Parse {
                    workbook: WORKBOOK,
                    sheet: SHEET,
                    row: index + 1,
                    column,
                    error,
                })?;

            if parsed_rows.contains_key(&id) {
                return Err(Error::DuplicateId {
                    type_name: std::any::type_name::<GenericSkill>(),
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

        unsafe { GENERIC_SKILL_TABLE.write(table); }

        for (id, row) in unsafe { GENERIC_SKILL_TABLE.assume_init_ref() }.rows.iter() {
            crate::skill::SkillTable::insert(&id, crate::skill::Skill::GenericSkill(row)).await?;
        }
        Ok(())
    }

    fn init() -> Result<(), Error> {
        (|| {
            for (id, row) in &mut unsafe { GENERIC_SKILL_TABLE.assume_init_mut() }.rows {
                for x in &mut row.effects {
                    x.init().map_err(|e| (*id, e))?;
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
