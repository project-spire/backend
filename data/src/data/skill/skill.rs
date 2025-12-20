// This is a generated file. DO NOT MODIFY.
#![allow(static_mut_refs)]

use crate::{DataId, error::Error};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use tokio::sync::Mutex;

static mut SKILL_TABLE: MaybeUninit<SkillTable> = MaybeUninit::uninit();

#[derive(Debug)]
pub enum Skill {
    GenericSkill(&'static crate::skill::GenericSkill),
    ScriptedSkill(&'static crate::skill::ScriptedSkill),
}

#[derive(Debug)]
pub struct SkillTable {
    rows: HashMap<DataId, Skill>,
}

impl Skill {
    pub fn id(&self) -> &DataId {
        match self {
            Self::GenericSkill(x) => &x.id,
            Self::ScriptedSkill(x) => &x.id,
        }
    }
}

impl crate::Linkable for Skill {
    fn get(id: &DataId) -> Option<&'static Self> {
        SkillTable::get(id)
    }
}

impl SkillTable {
    pub fn get(id: &DataId) -> Option<&'static Skill> {
        unsafe { &SKILL_TABLE.assume_init_ref().rows }.get(&id)
    }

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static Skill)> {
        unsafe { &SKILL_TABLE.assume_init_ref().rows }.iter()
    }

    pub(crate) fn init() {
        let table = Self { rows: HashMap::new() };
        unsafe { SKILL_TABLE.write(table); }
    }

    pub(crate) async fn insert(id: &DataId, row: Skill) -> Result<(), Error> {
        static LOCK: Mutex<()> = Mutex::const_new(());

        let rows = unsafe { &mut SKILL_TABLE.assume_init_mut().rows };
        let _ = LOCK.lock().await;

        if rows.contains_key(id) {
            return Err(Error::DuplicateId {
                type_name: std::any::type_name::<Skill>(),
                id: *id,
                a: format!("{:?}", rows[id]),
                b: format!("{:?}", row)
            });
        }
        rows.insert(*id, row);

        Ok(())
    }
}
