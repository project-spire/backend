use game_data::race::*;

static RACE_DATA: tokio::sync::OnceCell<RaceData> = tokio::sync::OnceCell::const_new();

pub async fn init() -> Result<(), ()> {
    RACE_DATA.get_or_init(|| {

    }).await;

    Ok(())
}
