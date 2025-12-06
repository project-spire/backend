// This is a generated file. DO NOT MODIFY.
#![allow(static_mut_refs)]

use std::collections::HashMap;
use std::mem::MaybeUninit;
use tokio::sync::Mutex;
use crate::{DataId, error::Error};

static mut EQUIPMENT_DATA: MaybeUninit<EquipmentData> = MaybeUninit::uninit();

#[derive(Debug)]
pub enum Equipment {
    Weapon(&'static crate::item::Weapon),
}

#[derive(Debug)]
pub struct EquipmentData {
    data: HashMap<DataId, Equipment>,
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
        EquipmentData::get(id)
    }
}

impl EquipmentData {
    pub fn get(id: &DataId) -> Option<&'static Equipment> {
        let data = unsafe { &EQUIPMENT_DATA.assume_init_ref().data };
        data.get(&id)
    }

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static Equipment)> {
        let data = unsafe { &EQUIPMENT_DATA.assume_init_ref().data };
        data.iter()
    }

    pub(crate) fn init() {
        let data = Self { data: HashMap::new() };
        unsafe { EQUIPMENT_DATA.write(data); }
    }

    pub(crate) async fn insert(id: &DataId, row: Equipment) -> Result<(), Error> {
        static LOCK: Mutex<()> = Mutex::const_new(());

        let data = unsafe { &mut EQUIPMENT_DATA.assume_init_mut().data };
        let _ = LOCK.lock().await;

        if data.contains_key(id) {
            return Err(Error::DuplicateId {
                type_name: std::any::type_name::<Equipment>(),
                id: *id,
                a: format!("{:?}", data[id]),
                b: format!("{:?}", row)
            });
        }
        data.insert(*id, row);

        crate::item::ItemData::insert(id, crate::item::Item::Equipment(&data[id])).await?;

        Ok(())
    }
}
