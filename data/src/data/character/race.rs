// This is a generated file. DO NOT MODIFY.
use crate::error::ParseError;
use crate::parse::*;
use std::io::Write;
use diesel::{AsExpression, FromSqlRow};
use diesel::deserialize::FromSql;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{IsNull, Output, ToSql};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(FromSqlRow, AsExpression)]
#[diesel(sql_type = crate::schema::sql_types::Race)]
pub enum Race {
    None,
    Human,
    Orc,
}

impl Race {
    pub fn parse(value: &calamine::Data) -> Result<Self, ParseError> {
        let enum_string = String::parse(value)?;

        Ok(match enum_string.as_str() {
            "None" => Self::None,
            "Human" => Self::Human,
            "Orc" => Self::Orc,
            _ => return Err(ParseError::InvalidValue {
                type_name: std::any::type_name::<Race>(),
                value: enum_string,
            }),
        })
    }

    pub fn try_from(value: &u16) -> Option<Self> {
        Some(match value {
            0 => Self::None,
            1 => Self::Human,
            2 => Self::Orc,
            _ => return None,
        })
    }
}

impl Into<u16> for Race {
    fn into(self) -> u16 {
        match self {
            Self::None => 0,
            Self::Human => 1,
            Self::Orc => 2,
        }
    }
}

impl Into<protocol::Race> for Race {
    fn into(self) -> protocol::Race {
        type Target = protocol::Race;

        match self {
            Self::None => Target::None,
            Self::Human => Target::Human,
            Self::Orc => Target::Orc,
        }
    }
}

impl Into<Race> for protocol::Race {
    fn into(self) -> Race {
        type Target = Race;

        match self {
            Self::None => Target::None,
            Self::Human => Target::Human,
            Self::Orc => Target::Orc,
        }
    }
}


impl ToSql<crate::schema::sql_types::Race, Pg> for Race {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
        match *self {
            Self::None => out.write_all(b"none")?,
            Self::Human => out.write_all(b"human")?,
            Self::Orc => out.write_all(b"orc")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::Race, Pg> for Race {
    fn from_sql(bytes: PgValue<'_>) -> diesel::deserialize::Result<Self> {
        Ok(match bytes.as_bytes() {
            b"none" => Self::None,
            b"human" => Self::Human,
            b"orc" => Self::Orc,
             _ => return Err(format!(
                "Unrecognized Race enum variant {}",
                String::from_utf8_lossy(bytes.as_bytes()),
            ).into()),
        })
    }
}
