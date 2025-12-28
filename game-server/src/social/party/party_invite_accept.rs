use super::PartyManager;
use actix::prelude::*;
use std::time::Instant;
use util::id::Id;

#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub struct PartyInviteAccept {
    pub party_id: Id,
    pub invitation_id: Id,
}

impl Handler<PartyInviteAccept> for PartyManager {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: PartyInviteAccept, _: &mut Self::Context) -> Self::Result {
        let PartyInviteAccept { party_id, invitation_id } = msg;
        
        let Some(party) = self.parties.get_mut(&party_id) else {
            return Err(());
        };
        
        let Some(invitation) = party.invitations.remove(&invitation_id) else {
            return Err(());
        };
        
        if /*TODO: Check if invitation is mine.*/ false {
            party.invitations.insert(invitation.id, invitation);
            return Err(());
        }
        
        if Instant::now() > invitation.expire_at {
            return Err(());
        }
        
        if party.members.len() >= party.member_capacity {
            return Err(());
        }
        
        party.members.insert(invitation.invitee);
        
        Ok(())
    }
}
