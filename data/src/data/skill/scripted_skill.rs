// This is a generated file. DO NOT MODIFY.
#![allow(static_mut_refs)]

use crate::{DataId, Link, error::*, parse::*};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use tracing::info;

const WORKBOOK: &str = "skill.ods";
const SHEET: &str = "ScriptedSkill";

static mut SCRIPTED_SKILL_TABLE: MaybeUninit<ScriptedSkillTable> = MaybeUninit::uninit();

#[derive(Debug)]
pub struct ScriptedSkill {
    pub id: DataId,
    pub attributes: serde_json::Value,
}

pub struct ScriptedSkillTable {
    rows: HashMap<DataId, ScriptedSkill>,
}

impl ScriptedSkill {
    fn parse(row: &[calamine::Data]) -> Result<(DataId, Self), (&'static str, ParseError)> {
        const FIELDS_COUNT: usize = 3;

        if row.len() < FIELDS_COUNT {
            return Err(("", ParseError::InvalidColumnCount { expected: FIELDS_COUNT, actual: row.len() }));
        }

        let id = parse(&row[0]).map_err(|e| ("id", e))?;
        let attributes = parse(&row[2]).map_err(|e| ("attributes", e))?;

        Ok((id, Self {
            id,
            attributes,
        }))
    }
}

impl crate::Linkable for ScriptedSkill {
    fn get(id: &DataId) -> Option<&'static Self> {
        ScriptedSkillTable::get(id)
    }
}

impl ScriptedSkillTable {
    pub fn get(id: &DataId) -> Option<&'static ScriptedSkill> {
        unsafe { SCRIPTED_SKILL_TABLE.assume_init_ref() }.rows.get(&id)
    }

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static ScriptedSkill)> {
        unsafe { SCRIPTED_SKILL_TABLE.assume_init_ref() }.rows.iter()
    }
}

impl crate::Loadable for ScriptedSkillTable {
    async fn load(rows: &[&[calamine::Data]]) -> Result<(), Error> {
        let mut parsed_rows = HashMap::new();
        let mut index = 2;

        for row in rows {
            let (id, parsed_row) = ScriptedSkill::parse(row)
                .map_err(|(column, error)| Error::Parse {
                    workbook: WORKBOOK,
                    sheet: SHEET,
                    row: index + 1,
                    column,
                    error,
                })?;

            if parsed_rows.contains_key(&id) {
                return Err(Error::DuplicateId {
                    type_name: std::any::type_name::<ScriptedSkill>(),
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

        unsafe { SCRIPTED_SKILL_TABLE.write(table); }

        for (id, row) in unsafe { SCRIPTED_SKILL_TABLE.assume_init_ref() }.rows.iter() {
            crate::skill::SkillTable::insert(&id, crate::skill::Skill::ScriptedSkill(row)).await?;
        }
        Ok(())
    }

    fn init() -> Result<(), Error> {
        Ok(())
    }
}
