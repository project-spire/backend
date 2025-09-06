use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn read_file(path: &Path) -> Result<String, std::io::Error> {
    let mut f = File::open(path)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;

    Ok(buf.trim().to_string())
}