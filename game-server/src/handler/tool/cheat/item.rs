use crate::task::Task;
use crate::world::zone::Zone;
use bevy_ecs::entity::Entity;
use protocol::game::tool::cheat_result::Result;

pub fn handle(entity: Entity, zone: &mut Zone, args: &[String]) -> Option<(Result, String)> {
    let task = Task::serial(async {
        let mut conn = db::conn().await?;

        //TODO: Insert a item

        Ok(())
    }).on_complete(|error, entity, world| {
        if let Some(error) = error {
            //TODO: Send fail message
            return;
        }
    });
    zone.dispatch_entity_task(entity, task);

    None
}
