// This is a generated file. DO NOT MODIFY.
#![allow(static_mut_refs)]

use std::collections::HashMap;
use std::mem::MaybeUninit;
use tracing::info;
use crate::{DataId, Link, error::*, parse::*};

static mut LINK_TEST_DATA: MaybeUninit<LinkTestData> = MaybeUninit::uninit();

#[derive(Debug)]
pub struct LinkTest {
    pub id: DataId,
    pub item_link: Link<crate::item::Item>,
    pub optional_item_link: Option<Link<crate::item::Item>>,
    pub multi_item_link: Vec<Link<crate::item::Item>>,
    pub exclusive_max5_item_link: Link<crate::item::Item>,
}

pub struct LinkTestData {
    data: HashMap<DataId, LinkTest>,
}

impl LinkTest {
    fn parse(row: &[calamine::Data]) -> Result<(DataId, Self), (&'static str, ParseError)> {
        const FIELDS_COUNT: usize = 6;

        if row.len() < FIELDS_COUNT {
            return Err(("", ParseError::InvalidColumnCount { expected: FIELDS_COUNT, actual: row.len() }));
        }

        let id = parse(&row[0]).map_err(|e| ("id", e))?;
        let item_link = parse(&row[1]).map_err(|e| ("item_link", e))?;
        let optional_item_link = parse_optional(&row[2]).map_err(|e| ("optional_item_link", e))?;
        let multi_item_link = parse_multi(&row[3]).map_err(|e| ("multi_item_link", e))?;
        let exclusive_max5_item_link = parse(&row[5]).map_err(|e| ("exclusive_max5_item_link", e))?;

        Ok((id, Self {
            id,
            item_link,
            optional_item_link,
            multi_item_link,
            exclusive_max5_item_link,
        }))
    }
}

impl crate::Linkable for LinkTest {
    fn get(id: &DataId) -> Option<&'static Self> {
        LinkTestData::get(id)
    }
}

impl LinkTestData {
    pub fn get(id: &DataId) -> Option<&'static LinkTest> {
        unsafe { LINK_TEST_DATA.assume_init_ref() }.data.get(&id)
    }

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static LinkTest)> {
        unsafe { LINK_TEST_DATA.assume_init_ref() }.data.iter()
    }
}

impl crate::Loadable for LinkTestData {
    async fn load(rows: &[&[calamine::Data]]) -> Result<(), Error> {
        let mut objects = HashMap::new();
        let mut index = 2;

        let mut exclusive_max5_item_link_set = std::collections::HashSet::<Link<crate::item::Item>>::new();

        let mut check_constraint = |object: &LinkTest| -> Result<(), (&'static str, ConstraintError)> {
            if !exclusive_max5_item_link_set.insert(object.exclusive_max5_item_link.clone()) {
                return Err(("exclusive_max5_item_link", ConstraintError::Unique {
                    type_name: std::any::type_name::<Link<crate::item::Item>>(),
                    value: object.exclusive_max5_item_link.to_string(),
                }));
            }

            if object.exclusive_max5_item_link > 5 {
                return Err(("exclusive_max5_item_link", ConstraintError::Max {
                    type_name: std::any::type_name::<Link<crate::item::Item>>(),
                    expected: 5.to_string(),
                    actual: object.exclusive_max5_item_link.to_string(),
                }));
            }

            Ok(())
        };

        for row in rows {
            let (id, object) = LinkTest::parse(row)
                .map_err(|(column, error)| Error::Parse {
                    workbook: "link_test.ods",
                    sheet: "LinkTest",
                    row: index + 1,
                    column,
                    error,
                })?;

            if objects.contains_key(&id) {
                return Err(Error::DuplicateId {
                    type_name: std::any::type_name::<LinkTest>(),
                    id,
                    a: format!("{:?}", objects[&id]),
                    b: format!("{:?}", object),
                });
            }

            check_constraint(&object)
                .map_err(|(column, error)| Error::Constraint {
                    workbook: "link_test.ods",
                    sheet: "LinkTest",
                    row: index + 1,
                    column,
                    error,
                })?;

            objects.insert(id, object);

            index += 1;
        }

        let data = Self { data: objects };
        unsafe { LINK_TEST_DATA.write(data); }

        info!("Loaded {} rows", rows.len());
        Ok(())
    }

    fn init() -> Result<(), Error> {
        (|| {
            for (id, row) in &mut unsafe { LINK_TEST_DATA.assume_init_mut() }.data {
                row.item_link.init().map_err(|e| (*id, e))?;

                if let Some(optional_item_link) = row.optional_item_link.as_mut() {
                    optional_item_link.init().map_err(|e| (*id, e))?;
                }

                for x in &mut row.multi_item_link {
                    x.init().map_err(|e| (*id, e))?;
                }

                row.exclusive_max5_item_link.init().map_err(|e| (*id, e))?;
            }

            Ok(())
        })().map_err(|(id, error)| Error::Link {
            workbook: "link_test.ods",
            sheet: "LinkTest",
            id,
            error,
        })?;

        Ok(())
    }
}
