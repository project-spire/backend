// This is a generated file. DO NOT MODIFY.
#![allow(static_mut_refs)]

use crate::{DataId, Link, error::*, parse::*};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use tracing::info;

const WORKBOOK: &str = "weapon.ods";
const SHEET: &str = "Weapon";

static mut WEAPON_TABLE: MaybeUninit<WeaponTable> = MaybeUninit::uninit();

#[derive(Debug)]
pub struct Weapon {
    pub id: DataId,
    pub name: String,
    pub weight: u16,
    pub damage: u32,
}

pub struct WeaponTable {
    rows: HashMap<DataId, Weapon>,
}

impl Weapon {
    fn parse(row: &[calamine::Data]) -> Result<(DataId, Self), (&'static str, ParseError)> {
        const FIELDS_COUNT: usize = 4;

        if row.len() < FIELDS_COUNT {
            return Err(("", ParseError::InvalidColumnCount { expected: FIELDS_COUNT, actual: row.len() }));
        }

        let id = parse(&row[0]).map_err(|e| ("id", e))?;
        let name = parse(&row[1]).map_err(|e| ("name", e))?;
        let weight = parse(&row[2]).map_err(|e| ("weight", e))?;
        let damage = parse(&row[3]).map_err(|e| ("damage", e))?;

        Ok((id, Self {
            id,
            name,
            weight,
            damage,
        }))
    }
}

impl crate::Linkable for Weapon {
    fn get(id: &DataId) -> Option<&'static Self> {
        WeaponTable::get(id)
    }
}

impl WeaponTable {
    pub fn get(id: &DataId) -> Option<&'static Weapon> {
        unsafe { WEAPON_TABLE.assume_init_ref() }.rows.get(&id)
    }

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static Weapon)> {
        unsafe { WEAPON_TABLE.assume_init_ref() }.rows.iter()
    }
}

impl crate::Loadable for WeaponTable {
    async fn load(rows: &[&[calamine::Data]]) -> Result<(), Error> {
        let mut parsed_rows = HashMap::new();
        let mut index = 2;

        for row in rows {
            let (id, parsed_row) = Weapon::parse(row)
                .map_err(|(column, error)| Error::Parse {
                    workbook: WORKBOOK,
                    sheet: SHEET,
                    row: index + 1,
                    column,
                    error,
                })?;

            if parsed_rows.contains_key(&id) {
                return Err(Error::DuplicateId {
                    type_name: std::any::type_name::<Weapon>(),
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

        unsafe { WEAPON_TABLE.write(table); }

        for (id, row) in unsafe { WEAPON_TABLE.assume_init_ref() }.rows.iter() {
            crate::item::EquipmentTable::insert(&id, crate::item::Equipment::Weapon(row)).await?;
        }
        Ok(())
    }

    fn init() -> Result<(), Error> {
        Ok(())
    }
}
