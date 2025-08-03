use crate::protocol::Error;

#[derive(Debug, PartialEq)]
pub enum Category {
    Auth = 1,
    Net = 2,
    Game = 3,
}

impl Category {
    pub fn decode(value: u8) -> Result<Self, Error> {
        Ok(match value {
            1 => Category::Auth,
            2 => Category::Net,
            3 => Category::Game,
            _ => return Err(Error::InvalidCategory(value)),
        })
    }
}
