// This is a generated file. DO NOT MODIFY.
#![allow(static_mut_refs)]

use crate::{DataId, error::Error};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use tokio::sync::Mutex;

static mut ITEM_TABLE: MaybeUninit<ItemTable> = MaybeUninit::uninit();

#[derive(Debug)]
pub enum Item {
    Equipment(&'static crate::item::Equipment),
    RandomBox(&'static crate::item::RandomBox),
}

#[derive(Debug)]
pub struct ItemTable {
    rows: HashMap<DataId, Item>,
}

impl Item {
    pub fn id(&self) -> &DataId {
        match self {
            Self::Equipment(x) => &x.id(),
            Self::RandomBox(x) => &x.id,
        }
    }
}

impl crate::Linkable for Item {
    fn get(id: &DataId) -> Option<&'static Self> {
        ItemTable::get(id)
    }
}

impl ItemTable {
    pub fn get(id: &DataId) -> Option<&'static Item> {
        unsafe { &ITEM_TABLE.assume_init_ref().rows }.get(&id)
    }

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static Item)> {
        unsafe { &ITEM_TABLE.assume_init_ref().rows }.iter()
    }

    pub(crate) fn init() {
        let table = Self { rows: HashMap::new() };
        unsafe { ITEM_TABLE.write(table); }
    }

    pub(crate) async fn insert(id: &DataId, row: Item) -> Result<(), Error> {
        static LOCK: Mutex<()> = Mutex::const_new(());

        let rows = unsafe { &mut ITEM_TABLE.assume_init_mut().rows };
        let _ = LOCK.lock().await;

        if rows.contains_key(id) {
            return Err(Error::DuplicateId {
                type_name: std::any::type_name::<Item>(),
                id: *id,
                a: format!("{:?}", rows[id]),
                b: format!("{:?}", row)
            });
        }
        rows.insert(*id, row);

        Ok(())
    }
}
