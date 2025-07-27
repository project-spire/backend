pub mod race;
pub mod race_stat;

use std::fs::File;
use std::io::BufReader;
use calamine::{open_workbook, Ods};
use std::path::PathBuf;

static FOO: tokio::sync::OnceCell<i32> = tokio::sync::OnceCell::const_new();


async fn GetFoo() -> i32 {
    FOO.get_or_init(|| async {
        1
    })
    .await;

    FOO.get().unwrap();
}

async fn load_table(path: PathBuf) -> Result<Ods<BufReader<File>>, calamine::Error> {
    let reader = open_workbook(path)?;
    Ok(reader)
}
