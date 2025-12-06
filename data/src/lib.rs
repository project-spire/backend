pub mod schema;
pub mod data;

pub use crate::data::*;
use std::cmp::Ordering;

mod error;
mod parse;

use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::mem::MaybeUninit;
use std::ops::Deref;

use crate::error::{Error, LinkError};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct DataId(u32);

#[derive(Debug)]
pub struct Link<T: Linkable + 'static> {
    id: DataId,
    target: MaybeUninit<&'static T>,
}

pub trait Linkable: Sized {
    fn get(id: &DataId) -> Option<&'static Self>;
}

pub(crate) trait Loadable: Sized {
    fn load(rows: &[&[calamine::Data]]) -> impl Future<Output = Result<(), Error>> + Send;
    fn init() -> Result<(), Error>;
}

impl From<i32> for DataId {
    fn from(value: i32) -> Self {
        Self(value as u32)
    }
}

impl From<u32> for DataId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Display for DataId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DataId({})", self.0)
    }
}

impl Deref for DataId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<u32> for DataId {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl<T: Linkable> Clone for Link<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            target: self.target,
        }
    }
}

impl<T: Linkable> Display for Link<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Link({:?})", self.id)
    }
}

impl<T: Linkable> Deref for Link<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.target.assume_init_ref() }
    }
}

impl<T: Linkable> PartialEq<u32> for Link<T> {
    fn eq(&self, other: &u32) -> bool {
        self.id == *other
    }
}

impl<T: Linkable> PartialOrd<u32> for Link<T> {
    fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
        self.id.partial_cmp(other)
    }
}

impl<T: Linkable + 'static> Link<T> {
    pub(crate) fn init(&mut self) -> Result<(), LinkError> {
        let target = match T::get(&self.id) {
            Some(target) => target,
            None => {
                return Err(LinkError::Missing {
                    type_name: std::any::type_name::<T>(),
                    id: self.id,
                });
            }
        };

        self.target.write(target);
        Ok(())
    }
}

impl<T: Linkable + 'static> Hash for Link<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T: Linkable + 'static> PartialEq for Link<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: Linkable + 'static> Eq for Link<T> {}
