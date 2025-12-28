mod party_create;
mod party_invite;
mod party_invite_accept;

pub use party_create::*;
pub use party_invite::*;
pub use party_invite_accept::*;

use actix::prelude::*;
use bevy_ecs::prelude::*;
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use util::id::Id;

pub struct Party {
    pub id: Id,
    pub name: Option<String>,
    pub master: Id,
    pub members: HashSet<Id>,
    pub invitations: HashMap<Id, PartyInvitation>,
    
    pub member_capacity: usize,
}

#[derive(Component)]
pub struct PartyMember {
    pub party_id: Id,
    pub is_master: bool,
}

pub struct PartyInvitation {
    pub id: Id,
    pub inviter: Id,
    pub invitee: Id,
    pub expire_at: Instant,
}

#[derive(Default)]
pub struct PartyManager {
    parties: HashMap<Id, Party>,
}

impl Actor for PartyManager {
    type Context = Context<Self>;
}

impl Supervised for PartyManager {}

impl SystemService for PartyManager {}
