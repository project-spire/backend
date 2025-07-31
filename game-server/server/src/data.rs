use calamine::DataType;
use std::ops::Deref;

include!(concat!(env!("OUT_DIR"), "/gen/data.rs"));

pub type DataId = u32;

#[derive(Debug)]
pub struct Link<'a, T> {
    id: DataId,
    target: &'a T,
}

impl<'a, T> Link<'a, T> {
    pub fn id(&self) -> DataId {
        self.id
    }
}

impl<'a, T> Deref for Link<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.target
    }
}

pub trait Linkable: Sized {
    fn get(id: DataId) -> Option<&'static Self>;
}

#[derive(Debug)]
pub enum LoadError {
    Workbook(calamine::OdsError),
    Sheet(calamine::Error),
    Parse(String),
    MissingLink { type_name: &'static str, id: DataId },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Workbook(e) => {
                write!(f, "{e}")
            },
            LoadError::Sheet(e) => {
                write!(f, "{e}")
            },
            LoadError::Parse(e) => {
                write!(f, "{e}")
            },
            LoadError::MissingLink { type_name, id } => {
                write!(f, "Missing link to {type_name} of id {id}")
            },
        }
    }
}

impl std::error::Error for LoadError {}

impl From<calamine::OdsError> for LoadError {
    fn from(value: calamine::OdsError) -> Self {
        LoadError::Workbook(value)
    }
}

impl From<calamine::Error> for LoadError {
    fn from(e: calamine::Error) -> Self {
        LoadError::Sheet(e)
    }
}

pub fn parse_id(value: &calamine::Data) -> Result<DataId, LoadError> {
    let i = match value.get_int() {
        Some(i) => i,
        None => return Err(LoadError::Parse(format!("{} is not Id", value.to_string()))),
    };

    if i < u32::MIN as i64 || i > u32::MAX as i64 {
        return Err(LoadError::Parse(format!("{} is out of Id range", value.to_string())));
    }

    Ok(i as u32)
}

pub fn parse_i8(value: &calamine::Data) -> Result<i8, LoadError> {
    let i = match value.get_int() {
        Some(i) => i,
        None => return Err(LoadError::Parse(format!("{} is not i8", value.to_string()))),
    };

    if i < i8::MIN as i64 || i > i8::MAX as i64 {
        return Err(LoadError::Parse(format!("{} is out of i8 range", value.to_string())));
    }

    Ok(i as i8)
}

pub fn parse_i16(value: &calamine::Data) -> Result<i16, LoadError> {
    let i = match value.get_int() {
        Some(i) => i,
        None => return Err(LoadError::Parse(format!("{} is not i16", value.to_string()))),
    };

    if i < i16::MIN as i64 || i > i16::MAX as i64 {
        return Err(LoadError::Parse(format!("{} is out of i16 range", value.to_string())));
    }

    Ok(i as i16)
}

pub fn parse_i32(value: &calamine::Data) -> Result<i32, LoadError> {
    let i = match value.get_int() {
        Some(i) => i,
        None => return Err(LoadError::Parse(format!("{} is not i32", value.to_string()))),
    };

    if i < i32::MIN as i64 || i > i32::MAX as i64 {
        return Err(LoadError::Parse(format!("{} is out of i32 range", value.to_string())));
    }

    Ok(i as i32)
}

pub fn parse_i64(value: &calamine::Data) -> Result<i64, LoadError> {
    let i = match value.get_int() {
        Some(i) => i,
        None => return Err(LoadError::Parse(format!("{} is not i64", value.to_string()))),
    };

    Ok(i)
}

pub fn parse_u8(value: &calamine::Data) -> Result<u8, LoadError> {
    let i = match value.get_int() {
        Some(i) => i,
        None => return Err(LoadError::Parse(format!("{} is not u8", value.to_string()))),
    };

    if i < u8::MIN as i64 || i > u8::MAX as i64 {
        return Err(LoadError::Parse(format!("{} is out of u8 range", value.to_string())));
    }

    Ok(i as u8)
}

pub fn parse_u16(value: &calamine::Data) -> Result<u16, LoadError> {
    let i = match value.get_int() {
        Some(i) => i,
        None => return Err(LoadError::Parse(format!("{} is not u16", value.to_string()))),
    };

    if i < u16::MIN as i64 || i > u16::MAX as i64 {
        return Err(LoadError::Parse(format!("{} is out of u16 range", value.to_string())));
    }

    Ok(i as u16)
}

pub fn parse_u32(value: &calamine::Data) -> Result<u32, LoadError> {
    let i = match value.get_int() {
        Some(i) => i,
        None => return Err(LoadError::Parse(format!("{} is not u32", value.to_string()))),
    };

    if i < u32::MIN as i64 || i > u32::MAX as i64 {
        return Err(LoadError::Parse(format!("{} is out of u32 range", value.to_string())));
    }

    Ok(i as u32)
}

pub fn parse_u64(value: &calamine::Data) -> Result<u64, LoadError> {
    let i = match value.get_int() {
        Some(i) => i,
        None => return Err(LoadError::Parse(format!("{} is not u64", value.to_string()))),
    };

    Ok(i as u64)
}

pub fn parse_f32(value: &calamine::Data) -> Result<f32, LoadError> {
    let f = match value.get_float() {
        Some(f) => f,
        None => return Err(LoadError::Parse(format!("{} is not f32", value.to_string()))),
    };

    Ok(f as f32)
}

pub fn parse_f64(value: &calamine::Data) -> Result<f64, LoadError> {
    let f = match value.get_float() {
        Some(f) => f,
        None => return Err(LoadError::Parse(format!("{} is not f64", value.to_string()))),
    };

    Ok(f)
}

pub fn parse_str(value: &calamine::Data) -> Result<String, LoadError> {
    let s = match value.get_string() {
        Some(s) => s,
        None => return Err(LoadError::Parse(format!("{} is not String", value.to_string()))),
    };

    Ok(s.to_owned())
}

pub fn parse_link<'a, T: Linkable + 'static>(value: &calamine::Data) -> Result<Link<'a, T>, LoadError> {
    let id = parse_id(value)?;
    let target = T::get(id).ok_or_else(|| LoadError::MissingLink {
        type_name: std::any::type_name::<T>(),
        id,
    })?;

    Ok(Link { id, target })
}
