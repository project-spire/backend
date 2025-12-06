// This is a generated file. DO NOT MODIFY.
#![allow(static_mut_refs)]

use std::collections::HashMap;
use std::mem::MaybeUninit;
use tokio::sync::Mutex;
use crate::{DataId, error::Error};

static mut ITEM_DATA: MaybeUninit<ItemData> = MaybeUninit::uninit();

#[derive(Debug)]
pub enum Item {
    Equipment(&'static crate::item::Equipment),
    RandomBox(&'static crate::item::RandomBox),
}

#[derive(Debug)]
pub struct ItemData {
    data: HashMap<DataId, Item>,
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
        ItemData::get(id)
    }
}

impl ItemData {
    pub fn get(id: &DataId) -> Option<&'static Item> {
        let data = unsafe { &ITEM_DATA.assume_init_ref().data };
        data.get(&id)
    }

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static Item)> {
        let data = unsafe { &ITEM_DATA.assume_init_ref().data };
        data.iter()
    }

    pub(crate) fn init() {
        let data = Self { data: HashMap::new() };
        unsafe { ITEM_DATA.write(data); }
    }

    pub(crate) async fn insert(id: &DataId, row: Item) -> Result<(), Error> {
        static LOCK: Mutex<()> = Mutex::const_new(());

        let data = unsafe { &mut ITEM_DATA.assume_init_mut().data };
        let _ = LOCK.lock().await;

        if data.contains_key(id) {
            return Err(Error::DuplicateId {
                type_name: std::any::type_name::<Item>(),
                id: *id,
                a: format!("{:?}", data[id]),
                b: format!("{:?}", row)
            });
        }
        data.insert(*id, row);

        Ok(())
    }
}
