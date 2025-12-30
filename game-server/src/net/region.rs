use crate::net::zone::Zone;
use actix::prelude::*;
use std::collections::HashMap;
use util::id::Id;

pub struct Region {
    pub id: Id,
    pub zones: HashMap<Id, Addr<Zone>>
}
