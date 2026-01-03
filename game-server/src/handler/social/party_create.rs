use actix::SystemService;
use crate::handler::ProtocolLocalHandler;
use crate::net::session::Session;
use crate::social::party::{PartyManager, PartyMember};
use crate::task::Task;
use bevy_ecs::prelude::*;
use tracing::error;
use protocol::game::social::{party_create_result, PartyCreate, PartyCreateResult};

impl ProtocolLocalHandler for PartyCreate {
    fn handle(self, world: &mut World, entity: Entity, session: Session) {
        use party_create_result::Error;

        // Check if already joined to a party.
        if world.get::<PartyMember>(entity).is_some() {
            session.send(&PartyCreateResult {
                error: Some(Error::Joined.into()),
                ..Default::default()
            });

            return;
        }

        let future = PartyManager::from_registry().send(crate::social::party::PartyCreate {
            requester_id: session.entry.character_id,
            name: self.name,
        });
        let task = Task::serial_with_return(future, move |result, world, entity| {
            let mut response = PartyCreateResult::default();

            let result = match result {
                Ok(result) => result,
                Err(e) => {
                    error!("Failed to create party: {}", e);

                    response.error = Some(Error::Internal.into());
                    session.send(&response);
                    return;
                }
            };
            
            match result {
                Ok(result) => {
                    response.party = Some(result.party);
                },
                Err(_) => {
                    // TODO: Use error type from result
                    response.error = Some(Error::Internal.into());
                    session.send(&response);
                    return;
                }
            }
        });

        task.dispatch(world, entity);
    }
}