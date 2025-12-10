// This is a generated file. DO NOT MODIFY.
#![allow(static_mut_refs)]

use crate::{DataId, error::Error};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use tokio::sync::Mutex;

static mut EQUIPMENT_TABLE: MaybeUninit<EquipmentTable> = MaybeUninit::uninit();

#[derive(Debug)]
pub enum Equipment {
    Weapon(&'static crate::item::Weapon),
}

#[derive(Debug)]
pub struct EquipmentTable {
    rows: HashMap<DataId, Equipment>,
}

impl Equipment {
    pub fn id(&self) -> &DataId {
        match self {
            Self::Weapon(x) => &x.id,
        }
    }
}

impl crate::Linkable for Equipment {
    fn get(id: &DataId) -> Option<&'static Self> {
        EquipmentTable::get(id)
    }
}

impl EquipmentTable {
    pub fn get(id: &DataId) -> Option<&'static Equipment> {
        unsafe { &EQUIPMENT_TABLE.assume_init_ref().rows }.get(&id)
    }

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static Equipment)> {
        unsafe { &EQUIPMENT_TABLE.assume_init_ref().rows }.iter()
    }

    pub(crate) fn init() {
        let table = Self { rows: HashMap::new() };
        unsafe { EQUIPMENT_TABLE.write(table); }
    }

    pub(crate) async fn insert(id: &DataId, row: Equipment) -> Result<(), Error> {
        static LOCK: Mutex<()> = Mutex::const_new(());

        let rows = unsafe { &mut EQUIPMENT_TABLE.assume_init_mut().rows };
        let _ = LOCK.lock().await;

        if rows.contains_key(id) {
            return Err(Error::DuplicateId {
                type_name: std::any::type_name::<Equipment>(),
                id: *id,
                a: format!("{:?}", rows[id]),
                b: format!("{:?}", row)
            });
        }
        rows.insert(*id, row);

        crate::item::ItemTable::insert(id, crate::item::Item::Equipment(&rows[id])).await?;

        Ok(())
    }
}
