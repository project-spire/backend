use super::{PartyInvitation, PartyManager};
use actix::prelude::*;
use util::id::Id;

#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub struct PartyInvite {
    pub party_id: Id,
    pub invitation: PartyInvitation,
}

impl Handler<PartyInvite> for PartyManager {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: PartyInvite, _: &mut Self::Context) -> Self::Result {
        let PartyInvite { party_id, invitation } = msg;

        let Some(party) = self.parties.get_mut(&party_id) else {
            return Err(());
        };

        party.invitations.insert(invitation.id, invitation);

        Ok(())
    }
}
