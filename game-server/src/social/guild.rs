use actix::prelude::*;
use bevy_ecs::prelude::*;
use std::collections::{HashMap, HashSet};
use util::id::Id;

pub struct Guild {
    pub id: Id,
    pub master: Id,
    pub members: HashSet<Id>,
}

#[derive(Component)]
pub struct GuildMember {
    pub guild_id: Id,
    pub rank: GuildMemberRank,
}

pub enum GuildMemberRank {
    Master,
    ViceMaster,
    Member,
}

#[derive(Default)]
pub struct GuildManager {
    guilds: HashMap<Id, Guild>,
}

impl Actor for GuildManager {
    type Context = Context<Self>;
}

impl Supervised for GuildManager {}

impl SystemService for GuildManager {}
