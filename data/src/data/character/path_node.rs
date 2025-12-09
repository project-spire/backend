// This is a generated file. DO NOT MODIFY.
use crate::{DataId, Link, error::*, parse::*};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use tracing::info;

const WORKBOOK: &str = "path_node.ods";
const SHEET: &str = "PathNode";

static PATH_NODE_DATA: MaybeUninit<PathNodeData> = MaybeUninit::uninit();

#[derive(Debug)]
pub struct PathNode {
    pub id: DataId,
}

pub struct PathNodeData {
    data: HashMap<DataId, PathNode>,
}

impl PathNode {
    fn parse(row: &[calamine::Data]) -> Result<(DataId, Self), (&'static str, ParseError)> {
        const FIELDS_COUNT: usize = 2;

        if row.len() < FIELDS_COUNT {
            return Err(("", ParseError::InvalidColumnCount { expected: FIELDS_COUNT, actual: row.len() }));
        }

        let id = parse(&row[0]).map_err(|e| ("id", e))?;

        Ok((id, Self {
            id,
        }))
    }
}

impl crate::Linkable for PathNode {
    fn get(id: &DataId) -> Option<&'static Self> {
        PathNodeData::get(id)
    }
}

impl PathNodeData {
    pub fn get(id: &DataId) -> Option<&'static PathNode> {
        unsafe { PATH_NODE_DATA.assume_init_ref() }.data.get(&id)
    }

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static PathNode)> {
        unsafe { PATH_NODE_DATA.assume_init_ref() }.data.iter()
    }
}

impl crate::Loadable for PathNodeData {
    async fn load(rows: &[&[calamine::Data]]) -> Result<(), Error> {
        let mut objects = HashMap::new();
        let mut index = 2;

        for row in rows {
            let (id, object) = PathNode::parse(row)
                .map_err(|(column, error)| Error::Parse {
                    workbook: WORKBOOK,
                    sheet: SHEET,
                    row: index + 1,
                    column,
                    error,
                })?;

            if objects.contains_key(&id) {
                return Err(Error::DuplicateId {
                    type_name: std::any::type_name::<PathNode>(),
                    id,
                    a: format!("{:?}", objects[&id]),
                    b: format!("{:?}", object),
                });
            }

            objects.insert(id, object);

            index += 1;
        }

        let data = Self { data: objects };
        unsafe { PATH_NODE_DATA.write(data); }

        info!("Loaded {} rows", rows.len());
        Ok(())
    }

    fn init() -> Result<(), Error> {
        Ok(())
    }
}
