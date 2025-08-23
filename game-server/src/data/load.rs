// This is a generated file. DO NOT MODIFY.
use calamine::Reader;

pub async fn load_all(data_dir: &std::path::PathBuf) -> Result<(), crate::data::LoadError> {
    type HandleType = tokio::task::JoinHandle<Result<(), crate::data::LoadError>>;

    fn add<T: crate::data::Loadable>(
        data_dir: &std::path::PathBuf,
        file_path: &str,
        sheet: &str,
        handles: &mut Vec<HandleType>,
    ) {
        let file_path = data_dir.join(file_path);
        let sheet = sheet.to_owned();

        handles.push(tokio::task::spawn(async move {
            let mut workbook: calamine::Ods<_> = calamine::open_workbook(file_path)?;
            let sheet = workbook
                .with_header_row(calamine::HeaderRow::Row(1))
                .worksheet_range(&sheet)?;
            T::load(&sheet.rows().collect::<Vec<_>>())?;

            Ok(())
        }));
    }

    async fn join(handles: Vec<HandleType>) -> Result<(), crate::data::LoadError> {
        for handle in handles {
            match handle.await {
                Ok(result) => result?,
                _  => panic!("Data loading task has failed!"),
            }
        }

        Ok(())
    }

    let mut level_0_handles = Vec::new();
    add::<crate::data::character::RaceStatData>(&data_dir, "character/race_stat.ods", "RaceStat", &mut level_0_handles);

    join(level_0_handles).await?;

    Ok(())
}
