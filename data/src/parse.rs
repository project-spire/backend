use crate::{DataId, Link, Linkable, error::ParseError};
use calamine::DataType;
use std::mem::MaybeUninit;
use std::str::FromStr;

pub(crate) trait Parsable: Sized {
    fn parse(value: &calamine::Data) -> Result<Self, ParseError>;

    fn parse_string(s: &str) -> Result<Self, ParseError>;
}

pub(crate) fn parse<T: Parsable>(value: &calamine::Data) -> Result<T, ParseError> {
    Ok(T::parse(value)?)
}

pub(crate) fn parse_optional<T: Parsable>(value: &calamine::Data) -> Result<Option<T>, ParseError> {
    if value.is_empty() {
        return Ok(None);
    }

    Ok(Some(T::parse(value)?))
}

pub(crate) fn parse_multi<T: Parsable>(value: &calamine::Data) -> Result<Vec<T>, ParseError> {
    let s = match value.get_string() {
        Some(s) => s.trim(),
        None => {
            return Err(ParseError::InvalidFormat {
                type_name: "array",
                expected: "string",
                actual: value.to_string(),
            });
        }
    }
    .trim();

    if s.len() < 2 || s.chars().nth(0).unwrap() != '[' || s.chars().nth_back(0).unwrap() != ']' {
        return Err(ParseError::InvalidFormat {
            type_name: "array",
            expected: "wrappers",
            actual: s.to_string(),
        });
    }

    let s = s[1..s.len() - 1].trim();
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    for c in s.chars() {
        match c {
            '(' => {
                depth += 1;
                current.push(c);
            }
            ')' => {
                depth -= 1;
                current.push(c);
            }
            ',' if depth == 0 => {
                result.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        result.push(current.trim().to_string());
    }

    Ok(result
        .iter()
        .map(|s| T::parse_string(&s))
        .collect::<Result<Vec<_>, _>>()?)
}

fn parse_tuple(value: &calamine::Data, count: usize) -> Result<Vec<&str>, ParseError> {
    let s = match value.get_string() {
        Some(s) => s.trim(),
        None => {
            return Err(ParseError::InvalidFormat {
                type_name: "tuple",
                expected: "string",
                actual: value.to_string(),
            });
        }
    }
    .trim();

    parse_tuple_string(s, count)
}

fn parse_tuple_string(s: &str, count: usize) -> Result<Vec<&str>, ParseError> {
    if s.len() < 2 || s.chars().nth(0).unwrap() != '(' || s.chars().nth_back(0).unwrap() != ')' {
        return Err(ParseError::InvalidFormat {
            type_name: "tuple",
            expected: "wrappers",
            actual: s.to_string(),
        });
    }

    let result: Vec<&str> = s[1..s.len() - 1]
        .trim()
        .split(",")
        .map(|item| item.trim())
        .collect();

    Ok(result)
}

fn to_integer<T>(value: &calamine::Data) -> Result<i64, ParseError> {
    match value.as_i64() {
        Some(i) => Ok(i),
        None => Err(ParseError::InvalidFormat {
            type_name: std::any::type_name::<T>(),
            expected: "integer",
            actual: value.to_string(),
        }),
    }
}

fn string_to_integer<T: FromStr>(s: &str) -> Result<T, ParseError> {
    let i = T::from_str(s).map_err(|_| ParseError::InvalidFormat {
        type_name: std::any::type_name::<T>(),
        expected: "integer",
        actual: s.to_string(),
    })?;

    Ok(i)
}

fn to_float<T>(value: &calamine::Data) -> Result<f64, ParseError> {
    match value.as_f64() {
        Some(f) => Ok(f),
        None => Err(ParseError::InvalidFormat {
            type_name: std::any::type_name::<T>(),
            expected: "float",
            actual: value.to_string(),
        }),
    }
}

fn string_to_float<T: FromStr>(s: &str) -> Result<T, ParseError> {
    let f = T::from_str(s).map_err(|_| ParseError::InvalidFormat {
        type_name: std::any::type_name::<T>(),
        expected: "float",
        actual: s.to_string(),
    })?;

    Ok(f)
}

fn check_range<T: Into<i64>>(value: i64, min: T, max: T) -> Result<(), ParseError> {
    let (min, max) = (min.into(), max.into());
    if min <= value && value <= max {
        return Ok(());
    }

    Err(ParseError::OutOfRange {
        type_name: std::any::type_name::<T>(),
        min,
        max,
        value,
    })
}

impl Parsable for DataId {
    fn parse(value: &calamine::Data) -> Result<Self, ParseError> {
        let i = to_integer::<DataId>(value)?;
        check_range(i, u32::MIN, u32::MAX)?;

        Ok(DataId(i as u32))
    }

    fn parse_string(s: &str) -> Result<Self, ParseError> {
        let id = string_to_integer::<u32>(s)?;
        Ok(DataId(id))
    }
}

impl Parsable for u8 {
    fn parse(value: &calamine::Data) -> Result<Self, ParseError> {
        let i = to_integer::<i8>(value)?;
        check_range(i, i8::MIN, i8::MAX)?;

        Ok(i as u8)
    }

    fn parse_string(s: &str) -> Result<Self, ParseError> {
        Ok(string_to_integer::<u8>(s)?)
    }
}

impl Parsable for u16 {
    fn parse(value: &calamine::Data) -> Result<Self, ParseError> {
        let i = to_integer::<u16>(value)?;
        check_range(i, u16::MIN, u16::MAX)?;

        Ok(i as u16)
    }

    fn parse_string(s: &str) -> Result<Self, ParseError> {
        Ok(string_to_integer::<u16>(s)?)
    }
}

impl Parsable for u32 {
    fn parse(value: &calamine::Data) -> Result<Self, ParseError> {
        let i = to_integer::<u32>(value)?;
        check_range(i, u32::MIN, u32::MAX)?;

        Ok(i as u32)
    }

    fn parse_string(s: &str) -> Result<Self, ParseError> {
        Ok(string_to_integer::<u32>(s)?)
    }
}

impl Parsable for u64 {
    fn parse(value: &calamine::Data) -> Result<Self, ParseError> {
        let i = to_integer::<u64>(value)?;

        Ok(i as u64)
    }

    fn parse_string(s: &str) -> Result<Self, ParseError> {
        Ok(string_to_integer::<u64>(s)?)
    }
}

impl Parsable for i8 {
    fn parse(value: &calamine::Data) -> Result<Self, ParseError> {
        let i = to_integer::<i8>(value)?;
        check_range(i, i8::MIN, i8::MAX)?;

        Ok(i as i8)
    }

    fn parse_string(s: &str) -> Result<Self, ParseError> {
        Ok(string_to_integer::<i8>(s)?)
    }
}

impl Parsable for i16 {
    fn parse(value: &calamine::Data) -> Result<Self, ParseError> {
        let i = to_integer::<i16>(value)?;
        check_range(i, i16::MIN, i16::MAX)?;

        Ok(i as i16)
    }

    fn parse_string(s: &str) -> Result<Self, ParseError> {
        Ok(string_to_integer::<i16>(s)?)
    }
}

impl Parsable for i32 {
    fn parse(value: &calamine::Data) -> Result<Self, ParseError> {
        let i = to_integer::<i32>(value)?;
        check_range(i, i32::MIN, i32::MAX)?;

        Ok(i as i32)
    }

    fn parse_string(s: &str) -> Result<Self, ParseError> {
        Ok(string_to_integer::<i32>(s)?)
    }
}

impl Parsable for i64 {
    fn parse(value: &calamine::Data) -> Result<Self, ParseError> {
        let i = to_integer::<i64>(value)?;

        Ok(i)
    }

    fn parse_string(s: &str) -> Result<Self, ParseError> {
        Ok(string_to_integer::<i64>(s)?)
    }
}

impl Parsable for f32 {
    fn parse(value: &calamine::Data) -> Result<Self, ParseError> {
        let f = to_float::<f32>(value)?;

        Ok(f as f32)
    }

    fn parse_string(s: &str) -> Result<Self, ParseError> {
        Ok(string_to_float::<f32>(s)?)
    }
}

impl Parsable for f64 {
    fn parse(value: &calamine::Data) -> Result<Self, ParseError> {
        let f = to_float::<f64>(value)?;

        Ok(f)
    }

    fn parse_string(s: &str) -> Result<Self, ParseError> {
        Ok(string_to_float::<f64>(s)?)
    }
}

impl Parsable for bool {
    fn parse(value: &calamine::Data) -> Result<Self, ParseError> {
        match value.get_bool() {
            Some(b) => Ok(b),
            None => Err(ParseError::InvalidFormat {
                type_name: std::any::type_name::<bool>(),
                expected: "bool",
                actual: value.to_string(),
            }),
        }
    }

    fn parse_string(s: &str) -> Result<Self, ParseError> {
        let b = bool::from_str(s).map_err(|_| ParseError::InvalidFormat {
            type_name: std::any::type_name::<bool>(),
            expected: "bool",
            actual: s.to_string(),
        })?;

        Ok(b)
    }
}

impl<T: 'static + Linkable> Parsable for Link<T> {
    fn parse(value: &calamine::Data) -> Result<Self, ParseError> {
        let id = DataId::parse(value)?;
        let target = MaybeUninit::uninit();

        Ok(Link { id, target })
    }

    fn parse_string(s: &str) -> Result<Self, ParseError> {
        let id = DataId::parse_string(s)?;
        let target = MaybeUninit::uninit();

        Ok(Link { id, target })
    }
}

impl Parsable for String {
    fn parse(value: &calamine::Data) -> Result<Self, ParseError> {
        let s = match value.get_string() {
            Some(s) => s,
            None => {
                return Err(ParseError::InvalidFormat {
                    type_name: std::any::type_name::<String>(),
                    expected: "string",
                    actual: value.to_string(),
                });
            }
        };

        Ok(s.trim().to_owned())
    }

    fn parse_string(s: &str) -> Result<Self, ParseError> {
        Ok(s.to_owned())
    }
}

impl<T1, T2> Parsable for (T1, T2)
where
    T1: Parsable,
    T2: Parsable,
{
    fn parse(value: &calamine::Data) -> Result<Self, ParseError> {
        let tuple = parse_tuple(value, 2)?;

        let first = T1::parse_string(tuple[0])?;
        let second = T2::parse_string(tuple[1])?;

        Ok((first, second))
    }

    fn parse_string(s: &str) -> Result<Self, ParseError> {
        let tuple = parse_tuple_string(s, 2)?;

        let first = T1::parse_string(tuple[0])?;
        let second = T2::parse_string(tuple[1])?;

        Ok((first, second))
    }
}

impl<T1, T2, T3> Parsable for (T1, T2, T3)
where
    T1: Parsable,
    T2: Parsable,
    T3: Parsable,
{
    fn parse(value: &calamine::Data) -> Result<Self, ParseError> {
        let tuple = parse_tuple(value, 3)?;

        let first = T1::parse_string(tuple[0])?;
        let second = T2::parse_string(tuple[1])?;
        let third = T3::parse_string(tuple[2])?;

        Ok((first, second, third))
    }

    fn parse_string(s: &str) -> Result<Self, ParseError> {
        let tuple = parse_tuple_string(s, 3)?;

        let first = T1::parse_string(tuple[0])?;
        let second = T2::parse_string(tuple[1])?;
        let third = T3::parse_string(tuple[2])?;

        Ok((first, second, third))
    }
}

impl<T1, T2, T3, T4> Parsable for (T1, T2, T3, T4)
where
    T1: Parsable,
    T2: Parsable,
    T3: Parsable,
    T4: Parsable,
{
    fn parse(value: &calamine::Data) -> Result<Self, ParseError> {
        let tuple = parse_tuple(value, 4)?;

        let first = T1::parse_string(tuple[0])?;
        let second = T2::parse_string(tuple[1])?;
        let third = T3::parse_string(tuple[2])?;
        let fourth = T4::parse_string(tuple[3])?;

        Ok((first, second, third, fourth))
    }

    fn parse_string(s: &str) -> Result<Self, ParseError> {
        let tuple = parse_tuple_string(s, 4)?;

        let first = T1::parse_string(tuple[0])?;
        let second = T2::parse_string(tuple[1])?;
        let third = T3::parse_string(tuple[2])?;
        let fourth = T4::parse_string(tuple[3])?;

        Ok((first, second, third, fourth))
    }
}
