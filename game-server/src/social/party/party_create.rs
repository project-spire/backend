use super::{Party, PartyManager};
use actix::prelude::*;
use std::collections::{HashMap, HashSet};
use util::id::Id;

#[derive(Message)]
#[rtype(result = "Result<PartyCreateResult, ()>")]
pub struct PartyCreate {
    pub requester_id: Id,
    pub name: Option<String>,
}

#[derive(MessageResponse)]
pub struct PartyCreateResult {
    pub party_id: Id,
}

impl Handler<PartyCreate> for PartyManager {
    type Result = Result<PartyCreateResult, ()>;

    fn handle(&mut self, msg: PartyCreate, _: &mut Self::Context) -> Self::Result {
        let PartyCreate { requester_id, name } = msg;
        
        // TODO: Check if already joined a party.

        // TODO: Check if can create a party.

        let party_id = util::id::global();
        let mut party = Party {
            id: party_id,
            name,
            master: requester_id,
            members: HashSet::new(),
            invitations: HashMap::new(),
            
            // TODO: Use data rather than hard-coded value.
            member_capacity: 10,
        };
        party.members.insert(msg.requester_id);

        self.parties.insert(party_id, party);

        let result = PartyCreateResult {
            party_id,
        };

        Ok(result)
    }
}
