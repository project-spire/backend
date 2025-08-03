//Generated code
#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub enum Category {
    Auth = 1,
    Net = 2,
    Game = 3,
}

impl Category {
    pub fn decode(value: u8) -> Result<Self, crate::protocol::Error> {
        Ok(match value {
            1 => Self::Auth,
            2 => Self::Net,
            3 => Self::Game,
            _ => return Err(crate::protocol::Error::InvalidCategory(value)),
        })
    }
}
