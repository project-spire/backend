// This is a generated file. DO NOT MODIFY.
use tracing::info;
use crate::data::DataId;

pub static RACE_STAT_DATA: tokio::sync::OnceCell<RaceStatData> = tokio::sync::OnceCell::const_new();

#[derive(Debug)]
pub struct RaceStat {
    pub id: DataId,
    pub race: crate::data::character::Race,
    pub speed: f32,
}

impl RaceStat {
    fn parse(row: &[calamine::Data]) -> Result<(DataId, Self), crate::data::LoadError> {
        let id = crate::data::parse_id(&row[0])?;
        let race = crate::data::character::Race::parse(&row[1])?;
        let speed = crate::data::parse_f32(&row[2])?;

        Ok((id, Self {
            id,
            race,
            speed,
        }))
    }
}

impl crate::data::Linkable for RaceStat {
    fn get(id: DataId) -> Option<&'static Self> {
        RaceStatData::get(id)
    }
}

pub struct RaceStatData {
    data: std::collections::HashMap<DataId, RaceStat>,
}

impl RaceStatData {
    pub fn get(id: DataId) -> Option<&'static RaceStat> {
        RACE_STAT_DATA
            .get()
            .expect("RACE_STAT_DATA is not initialized yet")
            .data
            .get(&id)
    }
}

impl crate::data::Loadable for RaceStatData {
    fn load(rows: &[&[calamine::Data]]) -> Result<(), crate::data::LoadError> {
        let mut objects = std::collections::HashMap::new();
        for row in rows {
            let (id, object) = RaceStat::parse(row)?;

            if objects.contains_key(&id) {
                return Err(crate::data::LoadError::DuplicatedId {
                    type_name: std::any::type_name::<RaceStat>(),
                    id,
                });
            }

            objects.insert(id, object);
        }

        if !RACE_STAT_DATA.set(Self { data: objects }).is_ok() {
            return Err(crate::data::LoadError::AlreadyLoaded {
                type_name: std::any::type_name::<RaceStat>(),
            });
        }

        info!("Loaded {} rows", rows.len());
        Ok(())
    }
}
