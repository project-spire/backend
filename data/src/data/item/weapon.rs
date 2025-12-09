// This is a generated file. DO NOT MODIFY.
use crate::{DataId, Link, error::*, parse::*};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use tracing::info;

const WORKBOOK: &str = "weapon.ods";
const SHEET: &str = "Weapon";

static WEAPON_DATA: MaybeUninit<WeaponData> = MaybeUninit::uninit();

#[derive(Debug)]
pub struct Weapon {
    pub id: DataId,
    pub name: String,
    pub weight: u16,
    pub damage: u32,
}

pub struct WeaponData {
    data: HashMap<DataId, Weapon>,
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
        WeaponData::get(id)
    }
}

impl WeaponData {
    pub fn get(id: &DataId) -> Option<&'static Weapon> {
        unsafe { WEAPON_DATA.assume_init_ref() }.data.get(&id)
    }

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static Weapon)> {
        unsafe { WEAPON_DATA.assume_init_ref() }.data.iter()
    }
}

impl crate::Loadable for WeaponData {
    async fn load(rows: &[&[calamine::Data]]) -> Result<(), Error> {
        let mut objects = HashMap::new();
        let mut index = 2;

        for row in rows {
            let (id, object) = Weapon::parse(row)
                .map_err(|(column, error)| Error::Parse {
                    workbook: WORKBOOK,
                    sheet: SHEET,
                    row: index + 1,
                    column,
                    error,
                })?;

            if objects.contains_key(&id) {
                return Err(Error::DuplicateId {
                    type_name: std::any::type_name::<Weapon>(),
                    id,
                    a: format!("{:?}", objects[&id]),
                    b: format!("{:?}", object),
                });
            }

            objects.insert(id, object);

            index += 1;
        }

        let data = Self { data: objects };
        unsafe { WEAPON_DATA.write(data); }

        for (id, row) in unsafe { WEAPON_DATA.assume_init_ref() }.data.iter() {
            crate::item::EquipmentData::insert(&id, crate::item::Equipment::Weapon(row)).await?;
        }

        info!("Loaded {} rows", rows.len());
        Ok(())
    }

    fn init() -> Result<(), Error> {
        Ok(())
    }
}
